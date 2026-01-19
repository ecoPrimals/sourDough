//! E2E test for RPC communication between primals.
//!
//! This test demonstrates the complete workflow:
//! 1. Start a primal RPC server
//! 2. Connect a client
//! 3. Execute RPC calls
//! 4. Verify responses

use sourdough_core::{
    health::{HealthReport, HealthStatus},
    identity::Did,
    lifecycle::PrimalState,
    rpc::{server::ServerConfig, PrimalRpc},
};
use std::net::SocketAddr;
use tarpc::{context, server::Channel, tokio_serde::formats::Bincode};
use tokio::net::TcpListener;

/// Mock primal that implements PrimalRpc
#[derive(Clone)]
struct MockPrimal {
    name: String,
    did: Did,
    state: PrimalState,
}

impl MockPrimal {
    fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            did: Did::parse("did:key:test123").unwrap(),
            state: PrimalState::Running,
        }
    }
}

#[tarpc::server]
impl PrimalRpc for MockPrimal {
    async fn health(self, _: context::Context) -> Result<HealthReport, String> {
        Ok(HealthReport::new(&self.name, "1.0.0")
            .with_status(HealthStatus::Healthy)
            .with_message("All systems operational"))
    }

    async fn state(self, _: context::Context) -> Result<PrimalState, String> {
        Ok(self.state)
    }

    async fn did(self, _: context::Context) -> Result<Did, String> {
        Ok(self.did)
    }

    async fn ping(self, _: context::Context) -> Result<String, String> {
        Ok("pong".to_string())
    }
}

#[tokio::test]
async fn test_rpc_server_client_communication() {
    // Create server with OS-assigned port
    let config = ServerConfig::default();
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();

    let listener = TcpListener::bind(addr).await.unwrap();
    let bound_addr = listener.local_addr().unwrap();

    println!("Server listening on: {}", bound_addr);

    // Create mock primal
    let primal = MockPrimal::new("test-primal");

    // Spawn server
    let server_handle = tokio::spawn(async move {
        loop {
            let (conn, _addr) = listener.accept().await.unwrap();
            let framed = tokio_serde::Framed::new(
                tokio_util::codec::Framed::new(conn, tokio_util::codec::LengthDelimitedCodec::new()),
                Bincode::default(),
            );
            let server = tarpc::server::BaseChannel::with_defaults(framed);
            tokio::spawn(server.execute(primal.clone().serve()));
        }
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Create client
    let conn = tokio::net::TcpStream::connect(bound_addr).await.unwrap();
    let framed = tokio_serde::Framed::new(
        tokio_util::codec::Framed::new(conn, tokio_util::codec::LengthDelimitedCodec::new()),
        Bincode::default(),
    );
    let client = PrimalRpcClient::new(tarpc::client::Config::default(), framed).spawn();

    // Test ping
    let response = client.ping(context::current()).await.unwrap();
    assert_eq!(response, "pong");
    println!("✓ Ping successful: {}", response);

    // Test DID retrieval
    let did = client.did(context::current()).await.unwrap();
    assert_eq!(did.to_string(), "did:key:test123");
    println!("✓ DID retrieved: {}", did);

    // Test state retrieval
    let state = client.state(context::current()).await.unwrap();
    assert_eq!(state, PrimalState::Running);
    println!("✓ State retrieved: {:?}", state);

    // Test health check
    let health = client.health(context::current()).await.unwrap();
    assert_eq!(health.status, HealthStatus::Healthy);
    assert_eq!(health.primal_name, "test-primal");
    println!("✓ Health check: {:?}", health.status);

    // Cleanup
    server_handle.abort();

    println!("\n✅ E2E RPC communication test passed!");
}

#[tokio::test]
async fn test_rpc_error_handling() {
    #[derive(Clone)]
    struct FailingPrimal;

    #[tarpc::server]
    impl PrimalRpc for FailingPrimal {
        async fn health(self, _: context::Context) -> Result<HealthReport, String> {
            Err("Service unavailable".to_string())
        }

        async fn state(self, _: context::Context) -> Result<PrimalState, String> {
            Err("Internal error".to_string())
        }

        async fn did(self, _: context::Context) -> Result<Did, String> {
            Err("Identity not available".to_string())
        }

        async fn ping(self, _: context::Context) -> Result<String, String> {
            Ok("pong".to_string()) // Ping always works
        }
    }

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bound_addr = listener.local_addr().unwrap();

    let primal = FailingPrimal;

    let server_handle = tokio::spawn(async move {
        loop {
            let (conn, _) = listener.accept().await.unwrap();
            let framed = tokio_serde::Framed::new(
                tokio_util::codec::Framed::new(conn, tokio_util::codec::LengthDelimitedCodec::new()),
                Bincode::default(),
            );
            let server = tarpc::server::BaseChannel::with_defaults(framed);
            tokio::spawn(server.execute(primal.clone().serve()));
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let conn = tokio::net::TcpStream::connect(bound_addr).await.unwrap();
    let framed = tokio_serde::Framed::new(
        tokio_util::codec::Framed::new(conn, tokio_util::codec::LengthDelimitedCodec::new()),
        Bincode::default(),
    );
    let client = PrimalRpcClient::new(tarpc::client::Config::default(), framed).spawn();

    // Ping should still work
    let response = client.ping(context::current()).await.unwrap();
    assert_eq!(response, "pong");
    println!("✓ Ping works even when other methods fail");

    // Health should fail gracefully
    match client.health(context::current()).await {
        Err(err) => {
            println!("✓ Health error handled: {:?}", err);
        }
        Ok(_) => panic!("Expected health to fail"),
    }

    server_handle.abort();

    println!("\n✅ E2E error handling test passed!");
}

#[tokio::test]
async fn test_rpc_concurrent_requests() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let bound_addr = listener.local_addr().unwrap();

    let primal = MockPrimal::new("concurrent-test");

    let server_handle = tokio::spawn(async move {
        loop {
            let (conn, _) = listener.accept().await.unwrap();
            let framed = tokio_serde::Framed::new(
                tokio_util::codec::Framed::new(conn, tokio_util::codec::LengthDelimitedCodec::new()),
                Bincode::default(),
            );
            let server = tarpc::server::BaseChannel::with_defaults(framed);
            tokio::spawn(server.execute(primal.clone().serve()));
        }
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let conn = tokio::net::TcpStream::connect(bound_addr).await.unwrap();
    let framed = tokio_serde::Framed::new(
        tokio_util::codec::Framed::new(conn, tokio_util::codec::LengthDelimitedCodec::new()),
        Bincode::default(),
    );
    let client = PrimalRpcClient::new(tarpc::client::Config::default(), framed).spawn();

    // Send 100 concurrent ping requests
    let mut handles = vec![];
    for i in 0..100 {
        let client_clone = client.clone();
        let handle = tokio::spawn(async move {
            let response = client_clone.ping(context::current()).await.unwrap();
            assert_eq!(response, "pong");
            i
        });
        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        handle.await.unwrap();
    }

    println!("✓ 100 concurrent requests handled successfully");

    server_handle.abort();

    println!("\n✅ E2E concurrent requests test passed!");
}

