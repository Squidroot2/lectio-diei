use std::error::Error;

use lectio_diei::date::DateId;
use lectio_diei::logging;
use lectio_diei::orchestration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    logging::init_logger()?;

    orchestration::retrieve_and_display(DateId::today()).await?;

    Ok(())
}
