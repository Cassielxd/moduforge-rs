use ahash::AHasher;
use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::rc::Rc;
use std::sync::Arc;
use thiserror::Error;

use crate::arena::UnsafeArena;
use crate::compiler::{Compiler, CompilerError};
use crate::expression::{Standard, Unary};
use crate::lexer::{Lexer, LexerError};
use crate::parser::{Parser, ParserError};
use crate::variable::Variable;
use crate::vm::{VMError, VM};
use crate::{Expression, ExpressionKind};

type ADefHasher = BuildHasherDefault<AHasher>;

/// Isolate 是一个组件，用于封装一个隔离的环境，用于执行表达式。
///
/// 重新运行 Isolate 允许通过 arena 分配器进行高效的内存重用。
/// arena 分配器通过重用内存块来优化内存管理，从而在 Isolate 被多次重用时提高性能和资源利用率。
///
/// 🆕 现在支持扩展机制和State集成
#[derive(Debug)]
pub struct Isolate<'arena> {
    lexer: Lexer<'arena>,
    compiler: Compiler,
    vm: VM,

    bump: UnsafeArena<'arena>,

    environment: Option<Variable>,
    references: HashMap<String, Variable, ADefHasher>,
}

impl<'a> Isolate<'a> {
    pub fn new() -> Self {
        Self {
            lexer: Lexer::new(),
            compiler: Compiler::new(),
            vm: VM::new(),

            bump: UnsafeArena::new(),

            environment: None,
            references: Default::default(),
        }
    }

    pub fn with_environment(variable: Variable) -> Self {
        let mut isolate = Isolate::new();
        isolate.set_environment(variable);

        isolate
    }

    pub fn set_environment(
        &mut self,
        variable: Variable,
    ) {
        self.environment.replace(variable);
    }

    pub fn update_environment<F>(
        &mut self,
        mut updater: F,
    ) where
        F: FnMut(Option<&mut Variable>),
    {
        updater(self.environment.as_mut());
    }

    pub fn set_reference(
        &mut self,
        reference: &'a str,
    ) -> Result<(), IsolateError> {
        let reference_value = match self.references.get(reference) {
            Some(value) => value.clone(),
            None => {
                let result = self.run_standard(reference)?;
                self.references.insert(reference.to_string(), result.clone());
                result
            },
        };

        if !matches!(&mut self.environment, Some(Variable::Object(_))) {
            self.environment.replace(Variable::empty_object());
        }

        let Some(Variable::Object(environment_object_ref)) = &self.environment
        else {
            return Err(IsolateError::ReferenceError);
        };

        let mut environment_object = environment_object_ref.borrow_mut();
        environment_object.insert(Rc::from("$"), reference_value);

        Ok(())
    }

    pub fn get_reference(
        &self,
        reference: &str,
    ) -> Option<Variable> {
        self.references.get(reference).cloned()
    }

    pub fn clear_references(&mut self) {
        self.references.clear();
    }

    fn run_internal(
        &mut self,
        source: &'a str,
        kind: ExpressionKind,
    ) -> Result<(), IsolateError> {
        self.bump.with_mut(|b| b.reset());
        let bump = self.bump.get();

        let tokens = self.lexer.tokenize(source)?;

        let base_parser = Parser::try_new(tokens, bump)?;
        let parser_result = match kind {
            ExpressionKind::Unary => base_parser.unary().parse(),
            ExpressionKind::Standard => base_parser.standard().parse(),
        };

        parser_result.error()?;

        self.compiler.compile(parser_result.root)?;

        Ok(())
    }

    pub fn compile_standard(
        &mut self,
        source: &'a str,
    ) -> Result<Expression<Standard>, IsolateError> {
        self.run_internal(source, ExpressionKind::Standard)?;
        let bytecode = self.compiler.get_bytecode().to_vec();

        Ok(Expression::new_standard(Arc::new(bytecode)))
    }

    pub fn run_standard(
        &mut self,
        source: &'a str,
    ) -> Result<Variable, IsolateError> {
        self.run_internal(source, ExpressionKind::Standard)?;

        let bytecode = self.compiler.get_bytecode();
        let result = self.vm.run(
            bytecode,
            self.environment.clone().unwrap_or(Variable::Null),
        )?;

        Ok(result)
    }

    /// 运行标准表达式，并传入State供自定义函数使用
    ///
    /// 使用 RAII 模式确保 State 的异常安全管理
    pub fn run_standard_with_state<S: Send + Sync + 'static>(
        &mut self,
        source: &'a str,
        state: Arc<S>,
    ) -> Result<Variable, IsolateError> {
        // 使用 StateGuard 自动管理 State 生命周期
        let _guard = crate::functions::StateGuard::new(state);

        // 运行表达式，即使发生异常，State 也会被正确清理
        self.run_standard(source)
    }

    pub fn compile_unary(
        &mut self,
        source: &'a str,
    ) -> Result<Expression<Unary>, IsolateError> {
        self.run_internal(source, ExpressionKind::Unary)?;
        let bytecode = self.compiler.get_bytecode().to_vec();

        Ok(Expression::new_unary(Arc::new(bytecode)))
    }

    pub fn run_unary(
        &mut self,
        source: &'a str,
    ) -> Result<bool, IsolateError> {
        self.run_internal(source, ExpressionKind::Unary)?;

        let bytecode = self.compiler.get_bytecode();
        let result = self.vm.run(
            bytecode,
            self.environment.clone().unwrap_or(Variable::Null),
        )?;

        result.as_bool().ok_or_else(|| IsolateError::ValueCastError)
    }

    /// 运行一元表达式，并传入State供自定义函数使用
    ///
    /// 使用 RAII 模式确保 State 的异常安全管理
    pub fn run_unary_with_state<S: Send + Sync + 'static>(
        &mut self,
        source: &'a str,
        state: Arc<S>,
    ) -> Result<bool, IsolateError> {
        // 使用 StateGuard 自动管理 State 生命周期
        let _guard = crate::functions::StateGuard::new(state);

        // 运行表达式，即使发生异常，State 也会被正确清理
        self.run_unary(source)
    }
}

/// Errors which happen within isolate or during evaluation
#[derive(Debug, Error)]
pub enum IsolateError {
    #[error("词法分析器错误: {source}")]
    LexerError { source: LexerError },

    #[error("解析器错误: {source}")]
    ParserError { source: ParserError },

    #[error("编译器错误: {source}")]
    CompilerError { source: CompilerError },

    #[error("虚拟机错误: {source}")]
    VMError { source: VMError },

    #[error("值转换错误")]
    ValueCastError,

    #[error("计算引用失败")]
    ReferenceError,

    #[error("缺少上下文引用")]
    MissingContextReference,
}

impl Serialize for IsolateError {
    fn serialize<S>(
        &self,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;

        match &self {
            IsolateError::ReferenceError => {
                map.serialize_entry("type", "referenceError")?;
            },
            IsolateError::MissingContextReference => {
                map.serialize_entry("type", "missingContextReference")?;
            },
            IsolateError::ValueCastError => {
                map.serialize_entry("type", "valueCastError")?;
            },
            IsolateError::LexerError { source } => {
                map.serialize_entry("type", "lexerError")?;
                map.serialize_entry("source", source.to_string().as_str())?;
            },
            IsolateError::ParserError { source } => {
                map.serialize_entry("type", "parserError")?;
                map.serialize_entry("source", source.to_string().as_str())?;
            },
            IsolateError::CompilerError { source } => {
                map.serialize_entry("type", "compilerError")?;
                map.serialize_entry("source", source.to_string().as_str())?;
            },
            IsolateError::VMError { source } => {
                map.serialize_entry("type", "vmError")?;
                map.serialize_entry("source", source.to_string().as_str())?;
            },
        }

        map.end()
    }
}

impl From<LexerError> for IsolateError {
    fn from(source: LexerError) -> Self {
        IsolateError::LexerError { source }
    }
}

impl From<ParserError> for IsolateError {
    fn from(source: ParserError) -> Self {
        IsolateError::ParserError { source }
    }
}

impl From<VMError> for IsolateError {
    fn from(source: VMError) -> Self {
        IsolateError::VMError { source }
    }
}

impl From<CompilerError> for IsolateError {
    fn from(source: CompilerError) -> Self {
        IsolateError::CompilerError { source }
    }
}
