// SPDX-License-Identifier: GPL-2.0-or-later

use razers_protocol_core::{REPORT_90_SIZE, Report90};
use razers_transport::{ReplayStep, ReplayTransport, ReportIo};

#[test]
fn replays_a_classic_request_response_without_hardware() {
    let request = Report90::command(0x00, 0x81, [0x00, 0x00])
        .unwrap()
        .with_transaction_id(0x1f);
    let request_bytes = request.encode().unwrap();

    let mut response = request.clone();
    response.status = 0x02;
    let response_bytes = response.encode().unwrap();

    let mut transport = ReplayTransport::new([
        ReplayStep::SetFeature {
            report_id: 0,
            payload: request_bytes.to_vec(),
        },
        ReplayStep::GetFeature {
            report_id: 0,
            response: response_bytes.to_vec(),
        },
    ]);

    transport.set_feature(0, &request_bytes).unwrap();
    let mut received = [0_u8; REPORT_90_SIZE];
    transport.get_feature(0, &mut received).unwrap();

    let decoded = Report90::decode(&received).unwrap();
    assert_eq!(decoded.status, 0x02);
    assert_eq!(decoded.transaction_id, request.transaction_id);
    assert_eq!(decoded.remaining_packets, request.remaining_packets);
    assert_eq!(decoded.command_class, request.command_class);
    assert_eq!(decoded.command_id, request.command_id);
    transport.verify_complete().unwrap();
}
