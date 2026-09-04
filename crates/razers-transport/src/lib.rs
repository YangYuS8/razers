// SPDX-License-Identifier: GPL-2.0-or-later

//! Byte-oriented transport boundaries for vendor HID reports.
//!
//! 厂商 HID 报文的字节级传输边界。
//!
//! # Example / 示例
//!
//! Validate a report exchange without opening any hardware.
//! 不打开硬件即可验证一次报文交换。
//!
//! ```
//! use razers_transport::{ReplayStep, ReplayTransport, ReportIo};
//!
//! let mut transport = ReplayTransport::new([
//!     ReplayStep::SetFeature { report_id: 0, payload: vec![1, 2] },
//!     ReplayStep::GetFeature { report_id: 0, response: vec![3, 4] },
//! ]);
//! transport.set_feature(0, &[1, 2])?;
//! let mut response = [0; 2];
//! assert_eq!(transport.get_feature(0, &mut response)?, 2);
//! assert_eq!(response, [3, 4]);
//! transport.verify_complete()?;
//! # Ok::<(), razers_transport::TransportError>(())
//! ```

use std::collections::VecDeque;

use thiserror::Error;

/// Synchronous, byte-level report I/O owned by one serialized connection worker.
///
/// Semantic operations such as setting DPI or lighting deliberately do not belong
/// in this trait. Platform backends are responsible for report-ID conventions.
///
/// 此 trait 由单个串行连接任务持有，只负责字节读写；DPI、灯光等语义操作不属于传输层。平台后端处理报文 ID 约定。
pub trait ReportIo: Send {
    /// Send a feature payload using the backend's report-ID convention.
    /// 按后端的报文 ID 约定发送 Feature 负载；错误不代表可以安全重试。
    fn set_feature(&mut self, report_id: u8, payload: &[u8]) -> Result<(), TransportError>;

    /// Read a feature response into the caller's buffer and return bytes copied.
    /// 将 Feature 响应写入调用方缓冲区并返回复制字节数；调用方需校验长度和协议内容。
    fn get_feature(&mut self, report_id: u8, output: &mut [u8]) -> Result<usize, TransportError>;

    /// Send an output report. Backends report I/O errors without semantic retries.
    /// 发送 Output 报告；后端返回 I/O 错误，不自行进行语义重试。
    fn write_output(&mut self, report_id: u8, payload: &[u8]) -> Result<(), TransportError>;

    /// Read an input report and return bytes copied; framing belongs to the backend.
    /// 读取 Input 报告并返回复制字节数；报文封装由后端处理。
    fn read_input(&mut self, output: &mut [u8]) -> Result<usize, TransportError>;
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum TransportError {
    #[error("transport I/O failed: {0}")]
    Io(String),
    #[error("replay trace ended before operation '{operation}'")]
    ReplayExhausted { operation: &'static str },
    #[error("replay expected '{expected}' but received '{actual}'")]
    OperationMismatch {
        expected: &'static str,
        actual: &'static str,
    },
    #[error("replay expected report ID 0x{expected:02x}, received 0x{actual:02x}")]
    ReportIdMismatch { expected: u8, actual: u8 },
    #[error("replay payload mismatch")]
    PayloadMismatch { expected: Vec<u8>, actual: Vec<u8> },
    #[error("output buffer is {available} bytes but the replay response requires {required}")]
    BufferTooSmall { required: usize, available: usize },
    #[error("replay completed with {remaining} unconsumed operations")]
    ReplayIncomplete { remaining: usize },
}

/// One expected operation in a deterministic, hardware-free transport trace.
///
/// 确定性、无硬件传输轨迹中的一个预期操作。
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplayStep {
    SetFeature { report_id: u8, payload: Vec<u8> },
    GetFeature { report_id: u8, response: Vec<u8> },
    WriteOutput { report_id: u8, payload: Vec<u8> },
    ReadInput { response: Vec<u8> },
}

impl ReplayStep {
    fn operation(&self) -> &'static str {
        match self {
            Self::SetFeature { .. } => "set_feature",
            Self::GetFeature { .. } => "get_feature",
            Self::WriteOutput { .. } => "write_output",
            Self::ReadInput { .. } => "read_input",
        }
    }
}

/// A strict in-memory transport for protocol tests and captured golden traces.
///
/// 用于协议测试和基准轨迹回放的严格内存传输实现。
#[derive(Clone, Debug, Default)]
pub struct ReplayTransport {
    steps: VecDeque<ReplayStep>,
}

impl ReplayTransport {
    /// Create a replay that consumes operations in order. 构建严格按序消费操作的回放。
    pub fn new(steps: impl IntoIterator<Item = ReplayStep>) -> Self {
        Self {
            steps: steps.into_iter().collect(),
        }
    }

    /// Number of operations not yet consumed. 尚未消费的操作数量。
    pub fn remaining(&self) -> usize {
        self.steps.len()
    }

    /// Reject incomplete traces with [`TransportError::ReplayIncomplete`].
    /// 未完成全部预期操作时返回错误，防止测试遗漏协议步骤。
    pub fn verify_complete(&self) -> Result<(), TransportError> {
        if self.steps.is_empty() {
            Ok(())
        } else {
            Err(TransportError::ReplayIncomplete {
                remaining: self.steps.len(),
            })
        }
    }

    fn next_for(&self, actual: &'static str) -> Result<ReplayStep, TransportError> {
        let step = self
            .steps
            .front()
            .ok_or(TransportError::ReplayExhausted { operation: actual })?
            .clone();
        let expected = step.operation();
        if expected != actual {
            return Err(TransportError::OperationMismatch { expected, actual });
        }
        Ok(step)
    }

    fn consume(&mut self) {
        self.steps.pop_front();
    }
}

impl ReportIo for ReplayTransport {
    fn set_feature(&mut self, report_id: u8, payload: &[u8]) -> Result<(), TransportError> {
        let ReplayStep::SetFeature {
            report_id: expected_id,
            payload: expected_payload,
        } = self.next_for("set_feature")?
        else {
            unreachable!("next_for validates the operation kind")
        };

        validate_report_id(expected_id, report_id)?;
        validate_payload(&expected_payload, payload)?;
        self.consume();
        Ok(())
    }

    fn get_feature(&mut self, report_id: u8, output: &mut [u8]) -> Result<usize, TransportError> {
        let ReplayStep::GetFeature {
            report_id: expected_id,
            response,
        } = self.next_for("get_feature")?
        else {
            unreachable!("next_for validates the operation kind")
        };

        validate_report_id(expected_id, report_id)?;
        copy_response(&response, output)?;
        self.consume();
        Ok(response.len())
    }

    fn write_output(&mut self, report_id: u8, payload: &[u8]) -> Result<(), TransportError> {
        let ReplayStep::WriteOutput {
            report_id: expected_id,
            payload: expected_payload,
        } = self.next_for("write_output")?
        else {
            unreachable!("next_for validates the operation kind")
        };

        validate_report_id(expected_id, report_id)?;
        validate_payload(&expected_payload, payload)?;
        self.consume();
        Ok(())
    }

    fn read_input(&mut self, output: &mut [u8]) -> Result<usize, TransportError> {
        let ReplayStep::ReadInput { response } = self.next_for("read_input")? else {
            unreachable!("next_for validates the operation kind")
        };

        copy_response(&response, output)?;
        self.consume();
        Ok(response.len())
    }
}

fn validate_report_id(expected: u8, actual: u8) -> Result<(), TransportError> {
    if expected == actual {
        Ok(())
    } else {
        Err(TransportError::ReportIdMismatch { expected, actual })
    }
}

fn validate_payload(expected: &[u8], actual: &[u8]) -> Result<(), TransportError> {
    if expected == actual {
        Ok(())
    } else {
        Err(TransportError::PayloadMismatch {
            expected: expected.to_vec(),
            actual: actual.to_vec(),
        })
    }
}

fn copy_response(response: &[u8], output: &mut [u8]) -> Result<(), TransportError> {
    if output.len() < response.len() {
        return Err(TransportError::BufferTooSmall {
            required: response.len(),
            available: output.len(),
        });
    }
    output[..response.len()].copy_from_slice(response);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replays_a_complete_feature_exchange() {
        let mut transport = ReplayTransport::new([
            ReplayStep::SetFeature {
                report_id: 0,
                payload: vec![1, 2, 3],
            },
            ReplayStep::GetFeature {
                report_id: 0,
                response: vec![4, 5, 6],
            },
        ]);

        transport.set_feature(0, &[1, 2, 3]).unwrap();
        let mut response = [0_u8; 3];
        assert_eq!(transport.get_feature(0, &mut response).unwrap(), 3);
        assert_eq!(response, [4, 5, 6]);
        assert_eq!(transport.verify_complete(), Ok(()));
    }

    #[test]
    fn mismatch_does_not_consume_the_trace() {
        let mut transport = ReplayTransport::new([ReplayStep::SetFeature {
            report_id: 0,
            payload: vec![1, 2, 3],
        }]);

        assert!(matches!(
            transport.set_feature(0, &[9]),
            Err(TransportError::PayloadMismatch { .. })
        ));
        assert_eq!(transport.remaining(), 1);
    }

    #[test]
    fn protects_small_output_buffers() {
        let mut transport = ReplayTransport::new([ReplayStep::ReadInput {
            response: vec![1, 2, 3],
        }]);
        let mut output = [0_u8; 2];

        assert_eq!(
            transport.read_input(&mut output),
            Err(TransportError::BufferTooSmall {
                required: 3,
                available: 2
            })
        );
        assert_eq!(transport.remaining(), 1);
    }
}
