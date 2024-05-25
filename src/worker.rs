use crate::tasks::health_check::add_post;
use celery::Celery;
use std::sync::Arc;

pub async fn create_worker() -> Arc<Celery> {
    let broker_url = std::env::var("BROKER_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379/".into());
    if broker_url.starts_with("redis"){
        return celery::app!(
            broker = test {broker_url},
            tasks = [
               add_post
            ],
            task_routes = [
                "add_post" => "test_queue"
            ],
            prefetch_count = 2,
            heartbeat = Some(10),
        ).await.unwrap()
    }
    return celery::app!(
        broker = AMQPBroker { broker_url },
        tasks = [
           add_post
        ],
        task_routes = [
            "add_post" => "test_queue"
        ],
        prefetch_count = 2,
        heartbeat = Some(10),
    ).await.unwrap()
   
}
