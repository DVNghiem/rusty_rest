use async_trait::async_trait;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::sync::Arc;

use crate::{AppError, AppResult};

/// Marker trait for requests that can be handled by the mediator
pub trait Request: Send + Sync + 'static {
    /// The response type associated with this request
    type Response: Send + Sync + 'static;
}

/// Trait for handlers that can process specific request types
#[async_trait]
pub trait Handler<TRequest>: Send + Sync
where
    TRequest: Request,
{
    /// Process the request and return the response
    async fn handle(&self, request: TRequest) -> AppResult<TRequest::Response>;
}

/// Main mediator trait for request processing
#[async_trait]
pub trait Mediator: Send + Sync {
    /// Send a request and get the response
    async fn send<TRequest>(&self, request: TRequest) -> AppResult<TRequest::Response>
    where
        TRequest: Request;
}

/// Type-erased handler wrapper for storage in the mediator
#[async_trait]
trait TypeErasedHandler: Send + Sync {
    async fn handle_erased(&self, request: Box<dyn Any + Send>) -> AppResult<Box<dyn Any + Send>>;
    fn request_type_id(&self) -> TypeId;
}

/// Wrapper to type-erase handlers
struct HandlerWrapper<TRequest, THandler>
where
    TRequest: Request,
    THandler: Handler<TRequest>,
{
    handler: THandler,
    _phantom: std::marker::PhantomData<TRequest>,
}

#[async_trait]
impl<TRequest, THandler> TypeErasedHandler for HandlerWrapper<TRequest, THandler>
where
    TRequest: Request,
    THandler: Handler<TRequest>,
{
    async fn handle_erased(&self, request: Box<dyn Any + Send>) -> AppResult<Box<dyn Any + Send>> {
        let request = *request
            .downcast::<TRequest>()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to downcast request")))?;

        let response = self.handler.handle(request).await?;
        Ok(Box::new(response))
    }

    fn request_type_id(&self) -> TypeId {
        TypeId::of::<TRequest>()
    }
}

/// Concrete mediator implementation that manages handlers and dispatches requests
pub struct ServiceMediator {
    handlers: HashMap<TypeId, Box<dyn TypeErasedHandler>>,
}

impl ServiceMediator {
    /// Create a new empty mediator
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a handler for a specific request type
    pub fn register_handler<TRequest, THandler>(
        &mut self,
        handler: THandler,
    ) -> AppResult<()>
    where
        TRequest: Request,
        THandler: Handler<TRequest> + 'static,
    {
        let type_id = TypeId::of::<TRequest>();
        
        if self.handlers.contains_key(&type_id) {
            return Err(AppError::Internal(anyhow::anyhow!(
                "Handler already registered for request type: {:?}",
                type_id
            )));
        }

        let wrapper = HandlerWrapper {
            handler,
            _phantom: std::marker::PhantomData,
        };

        self.handlers.insert(type_id, Box::new(wrapper));
        Ok(())
    }

    /// Check if a handler is registered for a request type
    pub fn has_handler<TRequest>(&self) -> bool
    where
        TRequest: Request,
    {
        self.handlers.contains_key(&TypeId::of::<TRequest>())
    }

    /// Get the number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }
}

impl Default for ServiceMediator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Mediator for ServiceMediator {
    async fn send<TRequest>(&self, request: TRequest) -> AppResult<TRequest::Response>
    where
        TRequest: Request,
    {
        let type_id = TypeId::of::<TRequest>();
        
        let handler = self.handlers.get(&type_id).ok_or_else(|| {
            AppError::Internal(anyhow::anyhow!(
                "No handler registered for request type: {:?}",
                type_id
            ))
        })?;

        let boxed_request = Box::new(request);
        let boxed_response = handler.handle_erased(boxed_request).await?;

        let response = *boxed_response
            .downcast::<TRequest::Response>()
            .map_err(|_| AppError::Internal(anyhow::anyhow!("Failed to downcast response")))?;

        Ok(response)
    }
}

/// Thread-safe mediator that can be shared across the application
pub type SharedMediator = Arc<dyn Mediator>;

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestRequest {
        pub value: i32,
    }

    impl Request for TestRequest {
        type Response = String;
    }

    struct TestHandler;

    #[async_trait]
    impl Handler<TestRequest> for TestHandler {
        async fn handle(&self, request: TestRequest) -> AppResult<String> {
            Ok(format!("Processed: {}", request.value))
        }
    }

    #[tokio::test]
    async fn test_mediator_basic_functionality() {
        let mut mediator = ServiceMediator::new();
        
        // Register handler
        mediator.register_handler::<TestRequest, _>(TestHandler).unwrap();
        
        // Send request
        let request = TestRequest { value: 42 };
        let response = mediator.send(request).await.unwrap();
        
        assert_eq!(response, "Processed: 42");
    }

    #[tokio::test]
    async fn test_mediator_no_handler_error() {
        let mediator = ServiceMediator::new();
        
        let request = TestRequest { value: 42 };
        let result = mediator.send(request).await;
        
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_duplicate_handler_registration() {
        let mut mediator = ServiceMediator::new();
        
        // Register handler
        mediator.register_handler::<TestRequest, _>(TestHandler).unwrap();
        
        // Try to register again
        let result = mediator.register_handler::<TestRequest, _>(TestHandler);
        assert!(result.is_err());
    }
}