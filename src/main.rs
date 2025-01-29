use std::path::PathBuf;
use std::str::FromStr;
use std::thread::sleep;

use futures_util::stream::StreamExt; // Para usar o StreamExt e o `next()`.
use grammers_client::session::Session;
use grammers_client::types::media::*;
use grammers_client::types::Message;
use grammers_client::{Client, Config, Update};

const ARQUIVOS_PERMITODOS: [&str; 3] = [
    "application/zip",
    "application/vnd.rar",
    "application/x-7z-compressed",
    
];
const PASTA_DOWNLOAD: &str = "Files";
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let session_path = "session";

    // Cria o cliente e tenta carregar a sessão
    let config = Config {
        session: Session::load_file_or_create(session_path)?,
        api_id:                                          // Sua API ID
        api_hash: "".to_string(), // Sua API Hash
        params: Default::default(),
    };

    let mut client = Client::connect(config).await?;
    println!("Conectado ao Telegram!");

    if !client.is_authorized().await? {
        println!("Você precisa se autenticar.");
        println!("Digite seu número de telefone (ex: +5511999999999): ");
        let mut phone_number = String::new();
        std::io::stdin().read_line(&mut phone_number)?;
        let phone_number = phone_number.trim();
        let token = client.request_login_code(phone_number).await?;
        println!("Um código foi enviado ao seu Telegram.");
        println!("Digite o código recebido (ex: 12345): ");
        let mut code = String::new();
        std::io::stdin().read_line(&mut code)?;
        client.sign_in(&token, code.trim()).await?;
        println!("Autenticação bem-sucedida!");
        println!("{:?}", client);
        client.session().save_to_file(session_path)?;
        println!("Sessão salva com sucesso!");
    } else {
        println!("Autenticação já concluída.");
    }

    loop {
        let update = client.next_update().await?;

        match update {
            Update::NewMessage(mensagem) => {
                if let Some(media) = mensagem.media() {
                    if let Media::Document(ref document) = media {
                        let name = document.name();
                        let v = document.mime_type().unwrap_or("foda");

                        if !ARQUIVOS_PERMITODOS.contains(&v) {
                            continue;
                        }

                      //  download_document_in_background(&client,   media.clone(), name).await;

                        let text = mensagem.text();
                        if let Some(passowrd) = extract_password(text) {
                            println!("Password: {}", passowrd);
                            download_document_in_background(&client,   media.clone(), name , passowrd).await;

                        } else {
                            download_document_in_background(&client,   media.clone(), name,  "password".to_string()).await;

                        }
                    }
                }
            }
            _ => {
                continue;
            }
        }
    }

    Ok(())
}

fn extract_password(text: &str) -> Option<String> {
    for text in text.lines() {
        let text = text.replace(" ", "");
        if text.contains("ssword:") {
            let line: Vec<&str> = text.split(":").collect();
            let len = line.len();
            if len > 1 {
                let password = line[1..].join(":");
                return Some(password.to_string());
            }
        }
    }
    None
}

use grammers_client::types::Downloadable;
use tokio::fs::create_dir_all;
use tokio::task;

async fn download_document_in_background(
    client: &Client, 
    message: Media, 
    file_name: &str, password:String
) {
   
    let mut path = PathBuf::from_str(PASTA_DOWNLOAD).unwrap();
    if !path.exists() {
        if let Err(e) = create_dir_all(&path).await {
            eprintln!("Erro ao criar diretório: {}", e);
            return;
        }
    }

    let full_path = path.join(file_name);

    let client_clone = client.clone();


    println!("Fazendo download");
    task::spawn(async move {
        let downloadable = Downloadable::Media(message);
        match client_clone.download_media(&downloadable, full_path.clone()).await {
            Ok(downloaded_file_path) => {
                // Rodando o comando `telegram_process.sh` em segundo plano
                // Substitua pela senha correta ou forneça dinamicamente 
                if let Err(e) = std::process::Command::new("telegram_process.sh")
                    .arg(&full_path) // Passa o caminho completo do arquivo como argumento
                    .arg(password)   // Passa a senha como argumento
                    .spawn()         // Executa o comando em segundo plano
                {
                    eprintln!("Erro ao executar o script telegram_process.sh: {}", e);
                } else {
                    println!("Script telegram_process.sh iniciado com sucesso.");
                }
            }
            Err(e) => {
                if let Err(e) = std::process::Command::new("telegram_process.sh")
                .arg(&full_path) // Passa o caminho completo do arquivo como argumento
                .arg(password)   // Passa a senha como argumento
                .spawn()         // Executa o comando em segundo plano
            {
                eprintln!("Erro ao executar o script telegram_process.sh: {}", e);
            } else {
                println!("Script telegram_process.sh iniciado com sucesso.");
            }
            
            }
        }
    });
    
}
