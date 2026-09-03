# Local IPC protocol

RazeRS protocol version 1 uses newline-delimited
[JSON-RPC 2.0](https://www.jsonrpc.org/specification) between the desktop application
and a private Agent child. Each line contains one request or response. Batch arrays
are not accepted by this transport version.

The application creates the Agent process and communicates through inherited pipes.
The Agent does not listen on a socket or network port, and it exits when the parent
closes its input. Release archives place `razers-agent` next to `razers`; development
builds can launch the same Agent entry point as a hidden child mode of the desktop
binary.

## Versioning

The JSON-RPC envelope always uses `"jsonrpc": "2.0"`. Every method parameter object
also carries `"protocol_version": 1`, and every successful result repeats that RazeRS
protocol version. JSON-RPC and RazeRS protocol versions are intentionally separate.

An unsupported RazeRS protocol version returns server error `-32001` with the
expected and received versions in `error.data`. Additive fields may be introduced
without changing the RazeRS version; incompatible method or field changes require a
new version.

## Methods

`agent.info` returns the Agent version, RazeRS protocol version, access mode, and
transport name. `devices.list` performs descriptor-only enumeration and returns
privacy-filtered device summaries. The current protocol exposes no method that opens
a device or sends a hardware report.

Example request:

```json
{"jsonrpc":"2.0","method":"devices.list","params":{"protocol_version":1},"id":1}
```

The Agent uses the standard JSON-RPC parse, invalid-request, method, parameter, and
internal error codes. A notification omits `id` and receives no response. Request IDs
may be strings, numbers, or null; the desktop client currently uses an integer.

## Privacy and future transports

Device paths and serial-number values are never included in IPC results. The current
transport has no ambient endpoint for another local process to discover. A persistent
per-user socket or named pipe is deferred until access control and peer verification
can be tested on every supported operating system.
