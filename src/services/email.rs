pub struct EmailService {
    _api_key: String,
    from_email: String,
}

impl EmailService {
    pub fn new(api_key: &str, from_email: String) -> Self {
        Self {
            _api_key: api_key.to_string(),
            from_email,
        }
    }

    pub async fn send_verification_email(
        &self,
        to_email: &str,
        verification_token: &str,
        base_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let verification_url = format!("{}/verify-email?token={}", base_url, verification_token);
        
        // For now, just log the email that would be sent
        println!(
            "📧 Would send verification email to {}\nFrom: {}\nSubject: Verify your email address\nLink: {}",
            to_email, self.from_email, verification_url
        );
        
        Ok(())
    }

    pub async fn send_password_reset_email(
        &self,
        to_email: &str,
        reset_token: &str,
        base_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let reset_url = format!("{}/reset-password?token={}", base_url, reset_token);
        
        // For now, just log the email that would be sent
        println!(
            "📧 Would send password reset email to {}\nFrom: {}\nSubject: Reset your password\nLink: {}",
            to_email, self.from_email, reset_url
        );
        
        Ok(())
    }

    pub async fn send_invitation_email(
        &self,
        to_email: &str,
        invitation_token: &str,
        base_url: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let invitation_url = format!("{}/accept-invitation?token={}", base_url, invitation_token);
        
        // For now, just log the email that would be sent
        println!(
            "📧 Would send invitation email to {}\nFrom: {}\nSubject: You're invited to join FreshAPI\nLink: {}",
            to_email, self.from_email, invitation_url
        );
        
        Ok(())
    }
}

impl Clone for EmailService {
    fn clone(&self) -> Self {
        Self {
            _api_key: self._api_key.clone(),
            from_email: self.from_email.clone(),
        }
    }
}