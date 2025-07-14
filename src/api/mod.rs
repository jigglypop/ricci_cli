use async_openai::{
    config::OpenAIConfig,
    types::{ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs, 
            ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
            ChatCompletionStreamResponseDelta},
    Client,
};
use anyhow::{Context, Result};
use futures::stream::StreamExt;
use tokio::sync::mpsc;
use crate::config::Config;

pub struct OpenAIClient {
    client: Client<OpenAIConfig>,
    model: String,
    temperature: f32,
    max_tokens: u16,
}

impl OpenAIClient {
    pub fn new(config: &Config) -> Result<Self> {
        let api_key = config.get_active_api_key()?;
        
        let openai_config = OpenAIConfig::new()
            .with_api_key(api_key);
        
        let client = Client::with_config(openai_config);
        
        Ok(Self {
            client,
            model: config.model_preferences.default_model.clone(),
            temperature: config.model_preferences.temperature,
            max_tokens: config.model_preferences.max_tokens,
        })
    }
    
    pub async fn query(&self, prompt: &str) -> Result<String> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a helpful development assistant.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(prompt)
                .build()?
                .into(),
        ];
        
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(messages)
            .temperature(self.temperature)
            .max_tokens(self.max_tokens)
            .build()?;
        
        let response = self.client
            .chat()
            .create(request)
            .await
            .context("OpenAI API 호출 실패")?;
        
        let content = response
            .choices
            .first()
            .and_then(|choice| choice.message.content.as_ref())
            .context("응답에서 콘텐츠를 찾을 수 없음")?;
        
        Ok(content.to_string())
    }
    
    pub async fn stream_chat(
        &self, 
        system_prompt: &str,
        messages: &[crate::assistant::Message]
    ) -> Result<mpsc::Receiver<Result<String>>> {
        let (tx, rx) = mpsc::channel(100);
        
        let mut chat_messages: Vec<ChatCompletionRequestMessage> = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
                .build()?
                .into(),
        ];
        
        // 기존 대화 기록 추가
        for msg in messages {
            let message = match msg.role.as_str() {
                "user" => ChatCompletionRequestUserMessageArgs::default()
                    .content(msg.content.clone())
                    .build()?
                    .into(),
                "assistant" => ChatCompletionRequestMessage::Assistant(
                    async_openai::types::ChatCompletionRequestAssistantMessage {
                        content: Some(msg.content.clone()),
                        ..Default::default()
                    }
                ),
                _ => continue,
            };
            chat_messages.push(message);
        }
        
        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(chat_messages)
            .temperature(self.temperature)
            .max_tokens(self.max_tokens)
            .stream(true)
            .build()?;
        
        let client = self.client.clone();
        
        // 스트리밍 태스크 생성
        tokio::spawn(async move {
            let mut stream = match client.chat().create_stream(request).await {
                Ok(s) => s,
                Err(e) => {
                    let _ = tx.send(Err(anyhow::anyhow!("스트림 생성 실패: {}", e))).await;
                    return;
                }
            };
            
            while let Some(result) = stream.next().await {
                match result {
                    Ok(response) => {
                        if let Some(choice) = response.choices.first() {
                            if let Some(ref delta) = choice.delta.content {
                                if tx.send(Ok(delta.clone())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        let _ = tx.send(Err(anyhow::anyhow!("스트림 오류: {}", e))).await;
                        break;
                    }
                }
            }
        });
        
        Ok(rx)
    }
}

// Re-export types that might be needed elsewhere
pub use crate::assistant::Message; 