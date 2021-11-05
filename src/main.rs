use tokio::fs;
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("creating output file.");
    fs::create_dir_all("./out").await?;

    println!("reading url file...");
    let urls = {
        let content = fs::read_to_string("./urls.txt").await?;
        content.split("\n")
            .map(|v| v.to_string())
            .collect::<Vec<String>>()
    };
    println!("got urls");

    println!("prepping {} urls", urls.len());
    let mut tasks = vec![];
    for url in urls {
        let handle = tokio::spawn(async move {
            let parsed = reqwest::Url::parse(&url)?;
            let filename = parsed.path()
                .rsplit( "/")
                .next()
                .unwrap();
            let resp = reqwest::get(url).await?;

            let fp = format!("./out/{}", filename);
            println!("creating file {}", &fp);

            let mut file = fs::OpenOptions::new()
                .write(true)
                .read(true)
                .create(true)
                .open(&fp)
                .await?;

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
        println!("task complete!")
    }

    Ok(())
}
