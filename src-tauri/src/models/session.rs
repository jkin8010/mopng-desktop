use image::DynamicImage;
use ort::{
    CPUExecutionProvider, CUDAExecutionProvider, CoreMLExecutionProvider,
    DirectMLExecutionProvider, ExecutionProviderDispatch, GraphOptimizationLevel, Session,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum OptLevel {
    Disable,
    Level1,
    Level2,
    Level3,
}

#[derive(Debug, Clone)]
pub struct SessionOptions {
    opt_level: Option<OptLevel>,
    num_threads: usize,
    parallel_execution: bool,
    memory_pattern: bool,
    providers: Option<Vec<String>>,
}

impl Default for SessionOptions {
    fn default() -> Self {
        Self {
            opt_level: Some(OptLevel::Level3),
            num_threads: 4,
            parallel_execution: true,
            memory_pattern: true,
            providers: Some(vec!["cpu".to_owned()]),
        }
    }
}

impl SessionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_opt_level(mut self, opt_level: OptLevel) -> Self {
        self.opt_level = Some(opt_level);
        self
    }

    pub fn with_num_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = num_threads;
        self
    }

    pub fn with_providers(mut self, providers: Vec<String>) -> Self {
        self.providers = Some(providers);
        self
    }

    pub fn build(&self) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(self.clone())
    }
}

pub struct BaseSession {
    pub inner_session: Session,
    pub model_path: String,
}

impl BaseSession {
    pub fn new(
        _debug: bool,
        session_options: SessionOptions,
        model_path: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("正在加载模型: {:?}", model_path);

        let mut session_builder = Session::builder()?;

        if let Some(opt_level) = session_options.opt_level {
            session_builder = session_builder.with_optimization_level(match opt_level {
                OptLevel::Disable => GraphOptimizationLevel::Disable,
                OptLevel::Level1 => GraphOptimizationLevel::Level1,
                OptLevel::Level2 => GraphOptimizationLevel::Level2,
                OptLevel::Level3 => GraphOptimizationLevel::Level3,
            })?;
        }

        session_builder = session_builder.with_intra_threads(
            if session_options.num_threads <= 0 { 1 } else { session_options.num_threads }
        )?;

        let providers = session_options
            .providers
            .unwrap_or(vec!["cpu".to_owned()])
            .iter()
            .map(|provider| match provider.as_str() {
                "coreml" => CoreMLExecutionProvider::default().build(),
                "cuda" => CUDAExecutionProvider::default().build(),
                "directml" => DirectMLExecutionProvider::default().build(),
                _ => CPUExecutionProvider::default().build(),
            })
            .collect::<Vec<ExecutionProviderDispatch>>();

        session_builder = session_builder.with_execution_providers(providers)?;

        let session = session_builder.commit_from_file(model_path.clone())?;
        log::info!("模型加载成功");

        Ok(Self {
            inner_session: session,
            model_path: model_path.to_string_lossy().to_string(),
        })
    }

    pub fn get_session(&self) -> Option<&Session> {
        Some(&self.inner_session)
    }
}

pub trait BaseSessionTrait {
    fn get_session(&self) -> Option<&Session>;

    fn run(
        &self,
        original_image: DynamicImage,
    ) -> Result<ndarray::Array3<u8>, Box<dyn std::error::Error>>;

    fn post_process(
        &self,
        output: ndarray::Array3<u8>,
        original_image: DynamicImage,
    ) -> Result<ndarray::Array3<u8>, Box<dyn std::error::Error>>;
}

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SessionError {
    #[error("Session not initialized")]
    PredictError,
    #[error("Model no output")]
    NoOutput,
    #[error("Image processing error")]
    ImageProcessingError,
    #[error("Model loading error")]
    ModelLoadError,
    #[error("Model not implemented")]
    NotImplemented,
}
