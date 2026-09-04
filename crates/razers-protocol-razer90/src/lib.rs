// SPDX-License-Identifier: GPL-2.0-or-later

//! Validated, synchronous exchanges for the classic 90-byte Razer protocol.
//!
//! This layer owns protocol timing and response validation while leaving byte
//! movement to [`ReportIo`]. A connection actor must retain exclusive ownership
//! of an exchange so one command cannot consume another command's response.
//!
//! 此层负责协议时序和响应校验，字节读写交给 ReportIo。连接任务必须独占一次交换，避免一条命令误取另一条命令的响应。

use std::{thread, time::Duration};

use razers_protocol_core::{REPORT_90_SIZE, Report90, Report90Error};
use razers_transport::{ReportIo, TransportError};
use thiserror::Error;

pub const STATUS_NEW_COMMAND: u8 = 0x00;
pub const STATUS_BUSY: u8 = 0x01;
pub const STATUS_SUCCESSFUL: u8 = 0x02;
pub const STATUS_FAILURE: u8 = 0x03;
pub const STATUS_TIMEOUT: u8 = 0x04;
pub const STATUS_NOT_SUPPORTED: u8 = 0x05;

/// How a device-specific driver interprets status `0x01`.
///
/// Existing implementations disagree: OpenRazer accepts a valid busy response
/// because some devices complete the operation anyway, while opsrzr resends the
/// request. RazeRS makes that choice explicit so write commands are never
/// retried accidentally.
///
/// 如何解释忙碌状态 0x01 由设备驱动决定。OpenRazer 接受有效忙碌响应，opsrzr 则重发；RazeRS 显式保留这种选择，防止写命令被意外重试。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BusyHandling {
    Accept,
    Retry,
}

/// Per-connection rules for one classic feature-report exchange.
///
/// 每个连接上一次经典功能报文交换所用的规则。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExchangePolicy {
    pub report_id: u8,
    pub response_wait: Duration,
    pub busy_handling: BusyHandling,
    pub busy_retries: u8,
    pub busy_retry_wait: Duration,
    pub validate_transaction_id: bool,
    pub validate_remaining_packets: bool,
}

impl ExchangePolicy {
    /// Construct a conservative policy that accepts, but does not resend, a
    /// valid busy response. Callers must provide their device's response delay.
    ///
    /// 构造保守策略：接受有效忙碌响应而不重发。调用者必须指定设备的响应等待时间。
    pub const fn new(report_id: u8, response_wait: Duration) -> Self {
        Self {
            report_id,
            response_wait,
            busy_handling: BusyHandling::Accept,
            busy_retries: 0,
            busy_retry_wait: Duration::from_millis(10),
            validate_transaction_id: false,
            validate_remaining_packets: true,
        }
    }

    /// Opt into resending a busy command. Use only for operations known to be
    /// safe to repeat on the selected device and firmware.
    ///
    /// 显式允许重发忙碌命令；仅适用于已确认在该设备与固件上可安全重复的操作。
    pub const fn with_busy_retry(mut self, retries: u8, wait: Duration) -> Self {
        self.busy_handling = BusyHandling::Retry;
        self.busy_retries = retries;
        self.busy_retry_wait = wait;
        self
    }

    pub const fn with_transaction_validation(mut self, enabled: bool) -> Self {
        self.validate_transaction_id = enabled;
        self
    }

    pub const fn with_remaining_packets_validation(mut self, enabled: bool) -> Self {
        self.validate_remaining_packets = enabled;
        self
    }
}

/// Injectable waiting boundary for deterministic protocol tests.
///
/// 可注入的等待接口，用于确定性协议测试。
pub trait Delay {
    fn wait(&mut self, duration: Duration);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ThreadDelay;

impl Delay for ThreadDelay {
    fn wait(&mut self, duration: Duration) {
        thread::sleep(duration);
    }
}

/// A serialized classic feature-report exchange over a byte transport.
///
/// 字节传输之上的串行经典功能报文交换。
pub struct FeatureExchange<T, D = ThreadDelay> {
    transport: T,
    policy: ExchangePolicy,
    delay: D,
}

impl<T> FeatureExchange<T, ThreadDelay> {
    pub fn new(transport: T, policy: ExchangePolicy) -> Self {
        Self {
            transport,
            policy,
            delay: ThreadDelay,
        }
    }
}

impl<T, D> FeatureExchange<T, D> {
    pub fn with_delay(transport: T, policy: ExchangePolicy, delay: D) -> Self {
        Self {
            transport,
            policy,
            delay,
        }
    }

    pub const fn policy(&self) -> &ExchangePolicy {
        &self.policy
    }

    pub fn into_parts(self) -> (T, D) {
        (self.transport, self.delay)
    }
}

impl<T: ReportIo, D: Delay> FeatureExchange<T, D> {
    /// Send one request and return a validated response.
    ///
    /// Command class and ID are always validated. Transaction ID and remaining
    /// packet validation are policy controlled because upstream devices differ.
    ///
    /// 发送一条请求并返回已校验响应。始终校验命令类与命令 ID；因上游设备存在差异，事务 ID 和剩余包数由策略决定是否校验。
    pub fn execute(&mut self, request: &Report90) -> Result<Report90, ExchangeError> {
        let request_bytes = request.encode()?;
        let maximum_attempts = usize::from(self.policy.busy_retries) + 1;

        for attempt in 1..=maximum_attempts {
            self.transport
                .set_feature(self.policy.report_id, &request_bytes)?;
            self.delay.wait(self.policy.response_wait);

            let mut response_bytes = [0_u8; REPORT_90_SIZE];
            let actual = self
                .transport
                .get_feature(self.policy.report_id, &mut response_bytes)?;
            if actual != REPORT_90_SIZE {
                return Err(ExchangeError::ShortRead {
                    expected: REPORT_90_SIZE,
                    actual,
                });
            }
            let response = Report90::decode(&response_bytes)?;
            self.validate_response(request, &response)?;

            match response.status {
                STATUS_SUCCESSFUL => return Ok(response),
                STATUS_BUSY if self.policy.busy_handling == BusyHandling::Accept => {
                    return Ok(response);
                }
                STATUS_BUSY if attempt < maximum_attempts => {
                    self.delay.wait(self.policy.busy_retry_wait);
                }
                STATUS_BUSY => return Err(ExchangeError::BusyExhausted { attempts: attempt }),
                status => {
                    return Err(ExchangeError::DeviceStatus {
                        status,
                        command_class: request.command_class,
                        command_id: request.command_id,
                    });
                }
            }
        }

        unreachable!("the inclusive attempt range always executes")
    }

    fn validate_response(
        &self,
        request: &Report90,
        response: &Report90,
    ) -> Result<(), ExchangeError> {
        if response.command_class != request.command_class
            || response.command_id != request.command_id
        {
            return Err(ExchangeError::EchoMismatch {
                expected_class: request.command_class,
                expected_id: request.command_id,
                actual_class: response.command_class,
                actual_id: response.command_id,
            });
        }
        if self.policy.validate_transaction_id && response.transaction_id != request.transaction_id
        {
            return Err(ExchangeError::TransactionMismatch {
                expected: request.transaction_id,
                actual: response.transaction_id,
            });
        }
        if self.policy.validate_remaining_packets
            && response.remaining_packets != request.remaining_packets
        {
            return Err(ExchangeError::RemainingPacketsMismatch {
                expected: request.remaining_packets,
                actual: response.remaining_packets,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum ExchangeError {
    #[error(transparent)]
    Transport(#[from] TransportError),
    #[error(transparent)]
    Codec(#[from] Report90Error),
    #[error("short feature response: expected {expected} bytes, received {actual}")]
    ShortRead { expected: usize, actual: usize },
    #[error(
        "response echo mismatch: expected class/id 0x{expected_class:02x}/0x{expected_id:02x}, received 0x{actual_class:02x}/0x{actual_id:02x}"
    )]
    EchoMismatch {
        expected_class: u8,
        expected_id: u8,
        actual_class: u8,
        actual_id: u8,
    },
    #[error("response transaction ID mismatch: expected 0x{expected:02x}, received 0x{actual:02x}")]
    TransactionMismatch { expected: u8, actual: u8 },
    #[error("response remaining-packets mismatch: expected {expected}, received {actual}")]
    RemainingPacketsMismatch { expected: u16, actual: u16 },
    #[error(
        "device returned status 0x{status:02x} for class/id 0x{command_class:02x}/0x{command_id:02x}"
    )]
    DeviceStatus {
        status: u8,
        command_class: u8,
        command_id: u8,
    },
    #[error("device remained busy after {attempts} attempts")]
    BusyExhausted { attempts: usize },
}

#[cfg(test)]
mod tests {
    use super::*;
    use razers_transport::{ReplayStep, ReplayTransport};

    #[derive(Default)]
    struct RecordingDelay {
        waits: Vec<Duration>,
    }

    impl Delay for RecordingDelay {
        fn wait(&mut self, duration: Duration) {
            self.waits.push(duration);
        }
    }

    fn request() -> Report90 {
        Report90::command(0x04, 0x85, [0x01])
            .unwrap()
            .with_transaction_id(0x3f)
    }

    fn response(request: &Report90, status: u8) -> Report90 {
        let mut response = Report90::command(request.command_class, request.command_id, [0x7f])
            .unwrap()
            .with_transaction_id(request.transaction_id);
        response.status = status;
        response.remaining_packets = request.remaining_packets;
        response
    }

    fn exchange_steps(request: &Report90, response: &Report90) -> [ReplayStep; 2] {
        [
            ReplayStep::SetFeature {
                report_id: 0,
                payload: request.encode().unwrap().to_vec(),
            },
            ReplayStep::GetFeature {
                report_id: 0,
                response: response.encode().unwrap().to_vec(),
            },
        ]
    }

    fn test_policy() -> ExchangePolicy {
        ExchangePolicy::new(0, Duration::from_millis(5)).with_transaction_validation(true)
    }

    #[test]
    fn completes_a_valid_exchange_without_real_waiting() {
        let request = request();
        let expected = response(&request, STATUS_SUCCESSFUL);
        let transport = ReplayTransport::new(exchange_steps(&request, &expected));
        let mut exchange =
            FeatureExchange::with_delay(transport, test_policy(), RecordingDelay::default());

        assert_eq!(exchange.execute(&request).unwrap(), expected);
        let (transport, delay) = exchange.into_parts();
        assert_eq!(transport.verify_complete(), Ok(()));
        assert_eq!(delay.waits, [Duration::from_millis(5)]);
    }

    #[test]
    fn can_accept_busy_without_resending_a_write() {
        let request = request();
        let expected = response(&request, STATUS_BUSY);
        let transport = ReplayTransport::new(exchange_steps(&request, &expected));
        let mut exchange =
            FeatureExchange::with_delay(transport, test_policy(), RecordingDelay::default());

        assert_eq!(exchange.execute(&request).unwrap(), expected);
        let (transport, delay) = exchange.into_parts();
        assert_eq!(transport.verify_complete(), Ok(()));
        assert_eq!(delay.waits, [Duration::from_millis(5)]);
    }

    #[test]
    fn retries_busy_when_the_policy_explicitly_allows_it() {
        let request = request();
        let busy = response(&request, STATUS_BUSY);
        let success = response(&request, STATUS_SUCCESSFUL);
        let steps = exchange_steps(&request, &busy)
            .into_iter()
            .chain(exchange_steps(&request, &success));
        let transport = ReplayTransport::new(steps);
        let policy = test_policy().with_busy_retry(2, Duration::from_millis(10));
        let mut exchange =
            FeatureExchange::with_delay(transport, policy, RecordingDelay::default());

        assert_eq!(exchange.execute(&request).unwrap(), success);
        let (transport, delay) = exchange.into_parts();
        assert_eq!(transport.verify_complete(), Ok(()));
        assert_eq!(
            delay.waits,
            [
                Duration::from_millis(5),
                Duration::from_millis(10),
                Duration::from_millis(5)
            ]
        );
    }

    #[test]
    fn reports_busy_retry_exhaustion() {
        let request = request();
        let busy = response(&request, STATUS_BUSY);
        let steps = (0..3).flat_map(|_| exchange_steps(&request, &busy));
        let transport = ReplayTransport::new(steps);
        let policy = test_policy().with_busy_retry(2, Duration::from_millis(10));
        let mut exchange =
            FeatureExchange::with_delay(transport, policy, RecordingDelay::default());

        assert_eq!(
            exchange.execute(&request),
            Err(ExchangeError::BusyExhausted { attempts: 3 })
        );
        let (transport, delay) = exchange.into_parts();
        assert_eq!(transport.verify_complete(), Ok(()));
        assert_eq!(delay.waits.len(), 5);
    }

    #[test]
    fn rejects_a_mismatched_command_echo() {
        let request = request();
        let mut wrong = response(&request, STATUS_SUCCESSFUL);
        wrong.command_id ^= 1;
        let transport = ReplayTransport::new(exchange_steps(&request, &wrong));
        let mut exchange =
            FeatureExchange::with_delay(transport, test_policy(), RecordingDelay::default());

        assert!(matches!(
            exchange.execute(&request),
            Err(ExchangeError::EchoMismatch { .. })
        ));
    }

    #[test]
    fn optionally_rejects_a_mismatched_transaction() {
        let request = request();
        let mut wrong = response(&request, STATUS_SUCCESSFUL);
        wrong.transaction_id = 0x1f;
        let transport = ReplayTransport::new(exchange_steps(&request, &wrong));
        let mut exchange =
            FeatureExchange::with_delay(transport, test_policy(), RecordingDelay::default());

        assert_eq!(
            exchange.execute(&request),
            Err(ExchangeError::TransactionMismatch {
                expected: 0x3f,
                actual: 0x1f
            })
        );
    }

    #[test]
    fn rejects_a_mismatched_packet_counter_by_default() {
        let request = request();
        let mut wrong = response(&request, STATUS_SUCCESSFUL);
        wrong.remaining_packets = 1;
        let transport = ReplayTransport::new(exchange_steps(&request, &wrong));
        let mut exchange =
            FeatureExchange::with_delay(transport, test_policy(), RecordingDelay::default());

        assert_eq!(
            exchange.execute(&request),
            Err(ExchangeError::RemainingPacketsMismatch {
                expected: 0,
                actual: 1
            })
        );
    }

    #[test]
    fn preserves_a_device_failure_status() {
        let request = request();
        let failed = response(&request, STATUS_NOT_SUPPORTED);
        let transport = ReplayTransport::new(exchange_steps(&request, &failed));
        let mut exchange =
            FeatureExchange::with_delay(transport, test_policy(), RecordingDelay::default());

        assert_eq!(
            exchange.execute(&request),
            Err(ExchangeError::DeviceStatus {
                status: STATUS_NOT_SUPPORTED,
                command_class: 0x04,
                command_id: 0x85
            })
        );
    }

    #[test]
    fn rejects_a_short_response_before_decoding() {
        let request = request();
        let transport = ReplayTransport::new([
            ReplayStep::SetFeature {
                report_id: 0,
                payload: request.encode().unwrap().to_vec(),
            },
            ReplayStep::GetFeature {
                report_id: 0,
                response: vec![0; REPORT_90_SIZE - 1],
            },
        ]);
        let mut exchange =
            FeatureExchange::with_delay(transport, test_policy(), RecordingDelay::default());

        assert_eq!(
            exchange.execute(&request),
            Err(ExchangeError::ShortRead {
                expected: REPORT_90_SIZE,
                actual: REPORT_90_SIZE - 1
            })
        );
    }
}
