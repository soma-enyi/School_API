#[cfg(test)]
mod email_template_tests {
    use crate::services::EmailTemplate;

    #[test]
    fn test_welcome_email_generation() {
        let user_name = "User";
        let role = "student";
        let html = EmailTemplate::welcome_email(user_name, role);
        
        assert!(html.contains(user_name));
        assert!(html.contains(role));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_welcome_email_with_different_role() {
        let user_name = "User";
        let role = "student";
        let html = EmailTemplate::welcome_email(user_name, role);
        
        assert!(html.contains(user_name));
        assert!(html.contains(role));
    }

    #[test]
    fn test_welcome_email_with_multiple_roles() {
        let roles = vec!["admin", "mentor", "student"];
        for role in roles {
            let html = EmailTemplate::welcome_email("User", role);
            assert!(html.contains(role));
        }
    }

    #[test]
    fn test_welcome_email_contains_role_badge() {
        let html = EmailTemplate::welcome_email("User", "admin");
        assert!(html.contains("role-badge"));
    }

    #[test]
    fn test_welcome_email_contains_header() {
        let html = EmailTemplate::welcome_email("User", "admin");
        assert!(html.contains("header"));
    }

    // ─── Application Pipeline Template Tests ───

    #[test]
    fn test_interview_invitation_template() {
        let html = EmailTemplate::interview_invitation("John Doe", "Solidity 101", "CourseFlow Campus, Lagos");
        assert!(html.contains("John Doe"));
        assert!(html.contains("Solidity 101"));
        assert!(html.contains("CourseFlow Campus, Lagos"));
        assert!(html.contains("Congratulations"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_waitlist_notification_template() {
        let html = EmailTemplate::waitlist_notification("Jane Doe", "Rust Basics");
        assert!(html.contains("Jane Doe"));
        assert!(html.contains("Rust Basics"));
        assert!(html.contains("Waitlist"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_enrollment_acceptance_template() {
        let html = EmailTemplate::enrollment_acceptance("Bob", "Web3 Dev", "TempPass123!", "https://courseflow.com/login");
        assert!(html.contains("Bob"));
        assert!(html.contains("Web3 Dev"));
        assert!(html.contains("TempPass123!"));
        assert!(html.contains("https://courseflow.com/login"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_rejection_email_template() {
        let html = EmailTemplate::rejection_email("Alice", "Data Science");
        assert!(html.contains("Alice"));
        assert!(html.contains("Data Science"));
        assert!(html.contains("apply again"));
        assert!(html.contains("<!DOCTYPE html>"));
    }

    #[test]
    fn test_pipeline_templates_contain_footer() {
        let templates = vec![
            EmailTemplate::interview_invitation("User", "Course", "Venue"),
            EmailTemplate::waitlist_notification("User", "Course"),
            EmailTemplate::enrollment_acceptance("User", "Course", "pass", "url"),
            EmailTemplate::rejection_email("User", "Course"),
        ];
        
        for template in templates {
            assert!(template.contains("footer"));
        }
    }

    #[test]
    fn test_pipeline_templates_contain_styles() {
        let templates = vec![
            EmailTemplate::interview_invitation("User", "Course", "Venue"),
            EmailTemplate::waitlist_notification("User", "Course"),
            EmailTemplate::enrollment_acceptance("User", "Course", "pass", "url"),
            EmailTemplate::rejection_email("User", "Course"),
        ];
        
        for template in templates {
            assert!(template.contains("<style>"));
        }
    }

    #[test]
    fn test_email_template_special_characters() {
        let special_chars = "<>&\"'";
        let html = EmailTemplate::welcome_email(special_chars, "admin");
        assert!(html.contains(special_chars));
    }

    #[test]
    fn test_pipeline_templates_html_structure() {
        let templates = vec![
            EmailTemplate::welcome_email("User", "admin"),
            EmailTemplate::interview_invitation("User", "Course", "Venue"),
            EmailTemplate::waitlist_notification("User", "Course"),
            EmailTemplate::enrollment_acceptance("User", "Course", "pass", "url"),
            EmailTemplate::rejection_email("User", "Course"),
        ];
        
        for template in templates {
            assert!(template.contains("<html>"));
            assert!(template.contains("</html>"));
            assert!(template.contains("<body>"));
            assert!(template.contains("</body>"));
        }
    }
}
