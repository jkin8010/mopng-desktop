use image::DynamicImage;
use std::path::PathBuf;
use tract_onnx::prelude::*;

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
    pub inner_session: SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>,
    pub model_path: String,
}

impl BaseSession {
    pub fn new(
        _debug: bool,
        session_options: SessionOptions,
        model_path: PathBuf,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        log::info!("正在加载模型: {:?}", model_path);

        let mut model = tract_onnx::onnx()
            .model_for_path(&model_path)?;

        // 设置输入形状 (batch=1, channels=3, height=1024, width=1024)
        model = model.with_input_fact(0, f32::fact([1, 3, 1024, 1024]).into())?;

        // 优化模型
        let model = model.into_optimized()?;

        // 编译为可运行模型
        let plan = model.into_runnable()?;

        log::info!("模型加载成功");

        Ok(Self {
            inner_session: plan,
            model_path: model_path.to_string_lossy().to_string(),
        })
    }
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
