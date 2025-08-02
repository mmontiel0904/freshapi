use serde_json;

pub struct EmailService {
    api_key: String,
    from_email: String,
    client: reqwest::Client,
}

impl EmailService {
    pub fn new(api_key: &str, from_email: String) -> Self {
        Self {
            api_key: api_key.to_string(),
            from_email,
            client: reqwest::Client::new(),
        }
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        verification_token: &str,
        base_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let verification_url = format!("{}/verify-email?token={}", base_url, verification_token);
        
        let html_content = format!(
            r#"
            <h2>Verify Your Email Address</h2>
            <p>Please click the link below to verify your email address:</p>
            <p><a href="{}">Verify Email</a></p>
            <p>If you didn't request this verification, please ignore this email.</p>
            "#,
            verification_url
        );

        self.send_email(
            to_email,
            "Verify your email address",
            &html_content,
        ).await
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_token: &str,
        base_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reset_url = format!("{}/reset-password?token={}", base_url, reset_token);
        
        let html_content = format!(
            r#"
            <h2>Reset Your Password</h2>
            <p>You requested to reset your password. Click the link below to proceed:</p>
            <p><a href="{}">Reset Password</a></p>
            <p>This link will expire in 1 hour.</p>
            <p>If you didn't request this reset, please ignore this email.</p>
            "#,
            reset_url
        );

        self.send_email(
            to_email,
            "Reset your password",
            &html_content,
        ).await
    }

    pub async fn send_invitation_email(
        &self,
        to_email: &str,
        invitation_token: &str,
        base_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let invitation_url = format!("{}/accept-invitation?token={}", base_url, invitation_token);
        
        let html_content = format!(
            r#"
            <h2>You're Invited to Join FreshAPI!</h2>
            <p>You've been invited to create an account on FreshAPI.</p>
            <p>Click the link below to accept your invitation and set up your account:</p>
            <p><a href="{}">Accept Invitation</a></p>
            <p>This invitation will expire in 7 days.</p>
            "#,
            invitation_url
        );

        self.send_email(
            to_email,
            "You're invited to join FreshAPI",
            &html_content,
        ).await
    }

    pub async fn send_admin_password_reset_email(
        &self,
        to_email: &str,
        reset_token: &str,
        base_url: &str,
        admin_name: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reset_url = format!("{}/reset-password?token={}", base_url, reset_token);
        
        let html_content = format!(
            r#"
            <h2>Password Reset Requested</h2>
            <p>An administrator ({}) has initiated a password reset for your account.</p>
            <p>Click the link below to set a new password:</p>
            <p><a href="{}">Reset Password</a></p>
            <p>This link will expire in 24 hours.</p>
            <p>If you have any concerns about this reset, please contact your administrator.</p>
            "#,
            admin_name,
            reset_url
        );

        self.send_email(
            to_email,
            "Password Reset - Admin Request",
            &html_content,
        ).await
    }

    async fn send_email(
        &self,
        to_email: &str,
        subject: &str,
        html_content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if we have a real API key or just the placeholder
        if self.api_key == "your-resend-api-key-here" || self.api_key == "dummy-key" {
            // Fallback to console logging for development
            println!(
                "📧 [DEVELOPMENT] Email would be sent via Resend:\nTo: {}\nFrom: {}\nSubject: {}\nContent: {}",
                to_email, self.from_email, subject, html_content
            );
            return Ok(());
        }

        let payload = serde_json::json!({
            "from": self.from_email,
            "to": [to_email],
            "subject": subject,
            "html": html_content
        });

        let response = self.client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Email sent successfully to {}", to_email);
        } else {
            let error_text = response.text().await?;
            return Err(format!("Failed to send email via Resend: {}", error_text).into());
        }

        Ok(())
    }
}

impl Clone for EmailService {
    fn clone(&self) -> Self {
        Self {
            api_key: self.api_key.clone(),
            from_email: self.from_email.clone(),
            client: self.client.clone(),
        }
    }
}