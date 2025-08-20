use sea_orm::*;
use uuid::Uuid;
use anyhow::Result;
use chrono::Utc;

use crate::entities::{
    project_context, email_context, project_context_category,
    prelude::*
};
use crate::graphql::types::{
    EmailIngestInput, EmailContextFilters, EmailContextConnection,
    AccountingProcess, ProcessingStatus
};
use crate::services::ContextService;

#[derive(Clone)]
pub struct EmailContextService {
    db: DatabaseConnection,
    context_service: ContextService,
}

impl EmailContextService {
    pub fn new(db: DatabaseConnection) -> Self {
        let context_service = ContextService::new(db.clone());
        Self { db, context_service }
    }

    pub fn get_db(&self) -> &DatabaseConnection {
        &self.db
    }

    // Email Ingestion - Main webhook endpoint logic
    pub async fn ingest_email(&self, input: EmailIngestInput) -> Result<email_context::Model> {
        let txn = self.db.begin().await?;

        // 1. Get email context type
        let email_context_type = self.context_service
            .get_context_type_by_name("email")
            .await?
            .ok_or_else(|| anyhow::anyhow!("Email context type not found"))?;

        // 2. Handle category (create if provided and doesn't exist)
        let category_id = if let Some(category_name) = &input.category_name {
            let existing_category = ProjectContextCategory::find()
                .filter(project_context_category::Column::ProjectId.eq(input.project_id))
                .filter(project_context_category::Column::ContextTypeId.eq(email_context_type.id))
                .filter(project_context_category::Column::Name.eq(category_name))
                .filter(project_context_category::Column::IsActive.eq(true))
                .one(&txn)
                .await?;

            match existing_category {
                Some(category) => Some(category.id),
                None => {
                    // Auto-create category
                    let new_category = project_context_category::ActiveModel {
                        id: Set(Uuid::new_v4()),
                        project_id: Set(input.project_id),
                        context_type_id: Set(email_context_type.id),
                        name: Set(category_name.clone()),
                        color: Set("#10b981".to_string()), // Green for auto-created email categories
                        description: Set(Some("Auto-created from email ingestion".to_string())),
                        is_active: Set(true),
                        created_by: Set(Uuid::new_v4()), // System user - could be configurable
                        created_at: Set(Utc::now().into()),
                        updated_at: Set(Utc::now().into()),
                    };

                    let created_category = new_category.insert(&txn).await?;
                    Some(created_category.id)
                }
            }
        } else {
            None
        };

        // 3. Check for duplicate email (by message_id if provided)
        if let Some(ref message_id) = input.message_id {
            let existing = EmailContext::find()
                .join(JoinType::InnerJoin, email_context::Relation::ProjectContext.def())
                .filter(project_context::Column::ProjectId.eq(input.project_id))
                .filter(email_context::Column::MessageId.eq(message_id))
                .one(&txn)
                .await?;

            if let Some(existing_email) = existing {
                // Email already exists, return it
                txn.commit().await?;
                return Ok(existing_email);
            }
        }

        // 4. Generate email title from subject and sender
        let title = if input.subject.len() > 100 {
            format!("{} - {}", &input.subject[..97], "...")
        } else {
            input.subject.clone()
        };

        let title = format!("{} (from {})", title, input.from_email);

        // 5. Create project context record
        let context_id = Uuid::new_v4();
        let project_context = project_context::ActiveModel {
            id: Set(context_id),
            project_id: Set(input.project_id),
            context_type_id: Set(email_context_type.id),
            category_id: Set(category_id),
            title: Set(title),
            description: Set(input.ai_summary.clone()),
            tags: Set(self.extract_tags_from_email(&input)),
            metadata: Set(Some(serde_json::json!({
                "ingestion_source": "n8n_webhook",
                "accounting_process": input.accounting_process.as_str(),
                "confidence_score": input.confidence_score,
                "has_attachments": input.has_attachments.unwrap_or(false),
            }))),
            is_archived: Set(false),
            created_by: Set(None), // From webhook
            created_at: Set(Utc::now().into()),
            updated_at: Set(Utc::now().into()),
        };

        project_context.insert(&txn).await?;

        // 6. Create email context record
        let email_context = email_context::ActiveModel {
            id: Set(context_id), // Same ID as project context
            from_email: Set(input.from_email),
            from_name: Set(input.from_name),
            to_emails: Set(input.to_emails),
            cc_emails: Set(input.cc_emails),
            bcc_emails: Set(input.bcc_emails),
            reply_to: Set(None), // Could be extracted from headers
            subject: Set(input.subject),
            message_preview: Set(input.message_preview.or_else(|| {
                // Auto-generate preview from full message
                if input.full_message.len() > 200 {
                    Some(format!("{}...", &input.full_message[..197]))
                } else {
                    Some(input.full_message.clone())
                }
            })),
            full_message: Set(input.full_message),
            message_html: Set(input.message_html),
            accounting_process: Set(input.accounting_process),
            ai_summary: Set(input.ai_summary),
            confidence_score: Set(input.confidence_score.map(|f| rust_decimal::Decimal::from_f64_retain(f).unwrap_or_default())),
            extracted_entities: Set(input.extracted_entities),
            message_id: Set(input.message_id),
            thread_id: Set(input.thread_id),
            in_reply_to: Set(input.in_reply_to),
            message_date: Set(input.message_date.map(|dt| dt.into())),
            received_date: Set(Utc::now().into()),
            has_attachments: Set(input.has_attachments.unwrap_or(false)),
            attachment_count: Set(input.attachment_count.unwrap_or(0)),
            processing_status: Set(ProcessingStatus::Completed.as_str().to_string()),
            processing_notes: Set(input.processing_notes),
        };

        let created_email = email_context.insert(&txn).await?;

        txn.commit().await?;
        Ok(created_email)
    }

    // Query email contexts with filters and pagination
    pub async fn get_email_contexts(
        &self,
        project_id: Uuid,
        filters: Option<EmailContextFilters>,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<EmailContextConnection> {
        let mut query = EmailContext::find()
            .join(JoinType::InnerJoin, email_context::Relation::ProjectContext.def())
            .filter(project_context::Column::ProjectId.eq(project_id))
            .filter(project_context::Column::IsArchived.eq(false));

        // Apply filters
        if let Some(filters) = filters {
            if let Some(accounting_process) = filters.accounting_process {
                query = query.filter(email_context::Column::AccountingProcess.eq(accounting_process));
            }

            if let Some(from_email) = filters.from_email {
                query = query.filter(email_context::Column::FromEmail.contains(&from_email));
            }

            if let Some(processing_status) = filters.processing_status {
                query = query.filter(email_context::Column::ProcessingStatus.eq(processing_status.as_str()));
            }

            if let Some(has_attachments) = filters.has_attachments {
                query = query.filter(email_context::Column::HasAttachments.eq(has_attachments));
            }

            if let Some(after_date) = filters.message_date_after {
                query = query.filter(email_context::Column::MessageDate.gte(after_date.naive_utc()));
            }

            if let Some(before_date) = filters.message_date_before {
                query = query.filter(email_context::Column::MessageDate.lte(before_date.naive_utc()));
            }

            // Full-text search on subject and message content
            if let Some(search_text) = filters.search_text {
                if !search_text.trim().is_empty() {
                    let search_condition = email_context::Column::Subject.contains(&search_text)
                        .or(email_context::Column::FullMessage.contains(&search_text))
                        .or(email_context::Column::AiSummary.contains(&search_text));
                    query = query.filter(search_condition);
                }
            }
        }

        // Get total count
        let total_count = query.clone().count(&self.db).await? as u32;

        // Apply pagination and ordering
        if let Some(limit) = limit {
            query = query.limit(limit);
        }
        if let Some(offset) = offset {
            query = query.offset(offset);
        }

        let emails = query
            .order_by_desc(email_context::Column::MessageDate)
            .order_by_desc(email_context::Column::ReceivedDate)
            .all(&self.db)
            .await?;

        Ok(EmailContextConnection {
            edges: emails.into_iter().map(Into::into).collect(),
            total_count,
        })
    }

    // Get email by ID
    pub async fn get_email_context_by_id(&self, email_id: Uuid) -> Result<Option<email_context::Model>> {
        EmailContext::find_by_id(email_id)
            .one(&self.db)
            .await
            .map_err(Into::into)
    }

    // Update processing status
    pub async fn update_processing_status(
        &self,
        email_id: Uuid,
        status: ProcessingStatus,
        notes: Option<String>,
    ) -> Result<email_context::Model> {
        let email = EmailContext::find_by_id(email_id)
            .one(&self.db)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Email context not found"))?;

        let mut email: email_context::ActiveModel = email.into();
        email.processing_status = Set(status.as_str().to_string());
        if let Some(notes) = notes {
            email.processing_notes = Set(Some(notes));
        }

        email.update(&self.db).await.map_err(Into::into)
    }

    // Search emails with full-text search
    pub async fn search_emails(
        &self,
        project_id: Uuid,
        search_query: &str,
        limit: Option<u64>,
    ) -> Result<Vec<email_context::Model>> {
        if search_query.trim().is_empty() {
            return Ok(Vec::new());
        }

        let query = EmailContext::find()
            .join(JoinType::InnerJoin, email_context::Relation::ProjectContext.def())
            .filter(project_context::Column::ProjectId.eq(project_id))
            .filter(project_context::Column::IsArchived.eq(false))
            .filter(
                email_context::Column::Subject.contains(search_query)
                    .or(email_context::Column::FullMessage.contains(search_query))
                    .or(email_context::Column::FromEmail.contains(search_query))
                    .or(email_context::Column::AiSummary.contains(search_query))
            )
            .order_by_desc(email_context::Column::MessageDate)
            .limit(limit.unwrap_or(50));

        query.all(&self.db).await.map_err(Into::into)
    }

    // Analytics: Get email stats by accounting process
    pub async fn get_email_stats_by_process(&self, project_id: Uuid) -> Result<std::collections::HashMap<AccountingProcess, u32>> {
        let stats = EmailContext::find()
            .join(JoinType::InnerJoin, email_context::Relation::ProjectContext.def())
            .filter(project_context::Column::ProjectId.eq(project_id))
            .filter(project_context::Column::IsArchived.eq(false))
            .group_by(email_context::Column::AccountingProcess)
            .column_as(email_context::Column::Id.count(), "count")
            .column(email_context::Column::AccountingProcess)
            .into_tuple::<(i64, AccountingProcess)>()
            .all(&self.db)
            .await?;

        Ok(stats.into_iter().map(|(count, process)| (process, count as u32)).collect())
    }

    // Helper function to extract tags from email content
    fn extract_tags_from_email(&self, input: &EmailIngestInput) -> Option<Vec<String>> {
        let mut tags = Vec::new();

        // Add accounting process as tag
        tags.push(input.accounting_process.as_str().to_string());

        // Add sender domain as tag
        if let Some(domain) = input.from_email.split('@').nth(1) {
            tags.push(format!("domain:{}", domain));
        }

        // Add attachment indicator
        if input.has_attachments.unwrap_or(false) {
            tags.push("has_attachments".to_string());
        }

        // Add confidence level tags
        if let Some(confidence) = input.confidence_score {
            if confidence >= 0.9 {
                tags.push("high_confidence".to_string());
            } else if confidence >= 0.7 {
                tags.push("medium_confidence".to_string());
            } else {
                tags.push("low_confidence".to_string());
            }
        }

        // Extract keywords from subject (simple keyword extraction)
        let subject_keywords = self.extract_keywords(&input.subject);
        tags.extend(subject_keywords);

        Some(tags)
    }

    // Simple keyword extraction from text
    fn extract_keywords(&self, text: &str) -> Vec<String> {
        let important_keywords = [
            "invoice", "payment", "receipt", "bill", "expense", "refund",
            "purchase", "order", "quote", "estimate", "contract", "tax",
            "report", "statement", "balance", "reconciliation", "audit"
        ];

        let text_lower = text.to_lowercase();
        important_keywords
            .iter()
            .filter(|&keyword| text_lower.contains(keyword))
            .map(|&keyword| format!("keyword:{}", keyword))
            .collect()
    }

    // Thread management
    pub async fn get_email_thread(&self, thread_id: &str, project_id: Uuid) -> Result<Vec<email_context::Model>> {
        EmailContext::find()
            .join(JoinType::InnerJoin, email_context::Relation::ProjectContext.def())
            .filter(project_context::Column::ProjectId.eq(project_id))
            .filter(email_context::Column::ThreadId.eq(thread_id))
            .order_by_asc(email_context::Column::MessageDate)
            .all(&self.db)
            .await
            .map_err(Into::into)
    }

    // Duplicate detection
    pub async fn find_potential_duplicates(
        &self,
        project_id: Uuid,
        from_email: &str,
        subject: &str,
        message_date: Option<chrono::DateTime<Utc>>,
    ) -> Result<Vec<email_context::Model>> {
        let mut query = EmailContext::find()
            .join(JoinType::InnerJoin, email_context::Relation::ProjectContext.def())
            .filter(project_context::Column::ProjectId.eq(project_id))
            .filter(email_context::Column::FromEmail.eq(from_email))
            .filter(email_context::Column::Subject.eq(subject));

        // If message date is provided, look for emails within 1 hour window
        if let Some(date) = message_date {
            let hour_before = date - chrono::Duration::hours(1);
            let hour_after = date + chrono::Duration::hours(1);
            query = query
                .filter(email_context::Column::MessageDate.gte(hour_before.naive_utc()))
                .filter(email_context::Column::MessageDate.lte(hour_after.naive_utc()));
        }

        query.all(&self.db).await.map_err(Into::into)
    }
}