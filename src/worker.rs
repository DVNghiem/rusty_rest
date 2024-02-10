use celery::Celery;
use std::sync::Arc;
use crate::tasks::health_check::add_post;

pub async fn create_worker() -> Arc<Celery> {
     let my_app = celery::app!(
        broker = RedisBroker { std::env::var("BROKER_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into()) },
        // broker = AMQPBroker { std::env::var("BROKER_URL").unwrap_or_else(|_| "amqp://127.0.0.1:5672".into()) },
        tasks = [
           add_post
        ],
        // This just shows how we can route certain tasks to certain queues based
        task_routes = [
            "add_post" => "test_queue"
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
    ).await.unwrap();
    my_app   
}