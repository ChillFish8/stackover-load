use tokio::fs;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let urls = {
        let content = fs::read_to_string("./target").await?;
        content.split("\n")
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
    };

    let mut tasks = vec![];
    for url in urls {
        let handle = tokio::spawn(async move {
            let parsed = reqwest::Url::parse(&url)?;
            let filename = parsed.path()
                .rsplitn(1, "/")
                .next()
                .unwrap();
            let resp = reqwest::get(url).await?;

            let mut file = fs::File::open(&format!("./{}", filename)).await?;

            let mut stream = resp.bytes_stream();
            while let Some(item) = stream.next().await {
                let mut buffer = item?;
                file.write_all_buf(&mut buffer).await?;
            }

            file.sync_all().await?;

            Ok::<_, anyhow::Error>(())
        });

        tasks.push(handle);
    }

    for task in tasks {
        task.await??;
    }

    Ok(())
}
