use mockall::predicate::*;
use mockall::*;

mock! {
    pub TwitterClient {
        fn login(&self) -> Result<()>;
        fn post_tweet(&self, text: String) -> Result<()>;
        // Add other methods you need to mock
    }
}

#[tokio::test]
async fn test_twitter_client() {
    let mut mock_client = MockTwitterClient::new();
    mock_client
        .expect_login()
        .times(1)
        .returning(|| Ok(()));
    
    mock_client
        .expect_post_tweet()
        .with(predicate::any())
        .times(1)
        .returning(|_| Ok(()));

    // Use mock client in your tests
    assert!(mock_client.login().is_ok());
} 