use crate::{
    async_read_le_ext::AsyncReadLeExt,
    protocol::{
        codec::{read_varchar, FeatureLevel},
        Context,
    },
};
use std::fmt;
use tokio::io::AsyncReadExt;

#[derive(Clone, Debug, thiserror::Error)]
pub struct TokenError {
    /// ErrorCode
    pub(crate) code: u32,
    /// ErrorState (describing code)
    pub(crate) state: u8,
    /// The class (severity) of the error
    pub(crate) class: u8,
    /// The error message
    pub(crate) message: String,
    pub(crate) server: String,
    pub(crate) procedure: String,
    pub(crate) line: u32,
}

impl TokenError {
    pub(crate) async fn decode<R>(src: &mut R, ctx: &Context) -> crate::Result<Self>
    where
        R: AsyncReadLeExt + Unpin,
    {
        let _length = src.read_u16_le().await? as usize;
        let code = src.read_u32_le().await?;
        let state = src.read_u8().await?;
        let class = src.read_u8().await?;

        let message_len = src.read_u16_le().await?;
        let message = read_varchar(src, message_len).await?;

        let server_len = src.read_u8().await?;
        let server = read_varchar(src, server_len).await?;

        let procedure_len = src.read_u8().await?;
        let procedure = read_varchar(src, procedure_len).await?;

        let line = if ctx.version > FeatureLevel::SqlServer2005 {
            src.read_u32_le().await?
        } else {
            src.read_u16_le().await? as u32
        };

        let token = TokenError {
            code,
            state,
            class,
            message,
            server,
            procedure,
            line,
        };

        Ok(token)
    }
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "'{}' on server {} executing {} on line {} (code: {}, state: {}, class: {})",
            self.message, self.server, self.procedure, self.line, self.code, self.state, self.class
        )
    }
}