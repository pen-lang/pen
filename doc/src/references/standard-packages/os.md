# `Os` package

## `Os'Context` module

### Types

#### `Context`

```pen
type Context {
  ...
}
```

#### `InnerContext`

```pen
type InnerContext {
  ReadStdIn \() FfiStringResult
  ReadLimitStdIn \(number) FfiStringResult
  WriteStdOut \(string) FfiNumberResult
  WriteStdErr \(string) FfiNumberResult
  OpenFile \(string, OpenOptions) FfiOpenResult
  ReadFile \(NormalFile) FfiStringResult
  ReadLimitFile \(NormalFile, number) FfiStringResult
  WriteFile \(NormalFile, string) FfiNumberResult
  CopyFile \(string, string) FfiNoneResult
  MoveFile \(string, string) FfiNoneResult
  RemoveFile \(string) FfiNoneResult
  ReadDirectory \(string) FfiReadDirectoryResult
  CreateDirectory \(string) FfiNoneResult
  RemoveDirectory \(string) FfiNoneResult
  GetArguments \() array'Array
  GetEnvironmentVariable \(string) FfiStringResult
  Metadata \(string) FfiMetadataResult
  TcpBind \(string) FfiTcpListenerResult
  TcpConnect \(string) FfiTcpStreamResult
  TcpAccept \(Tcp'Listener) FfiTcpAcceptedStreamResult
  TcpReceive \(Tcp'Stream, number) FfiStringResult
  TcpSend \(Tcp'Stream, string) FfiNumberResult
  UdpBind \(string) FfiUdpSocketResult
  UdpConnect \(Udp'Socket, string) FfiNoneResult
  UdpReceive \(Udp'Socket) FfiStringResult
  UdpReceiveFrom \(Udp'Socket) FfiUdpDatagramResult
  UdpSend \(Udp'Socket, string) FfiNumberResult
  UdpSendTo \(Udp'Socket, string, string) FfiNumberResult
  GetTime \() number
  Sleep \(number) none
  RandomNumber \() number
  Exit \(number) none
}
```

#### `FfiOpenResult`

```pen
type FfiOpenResult {
  File NormalFile
  Error string
}
```

#### `FfiStringResult`

```pen
type FfiStringResult {
  Value string
  Error string
}
```

#### `FfiNumberResult`

```pen
type FfiNumberResult {
  Value number
  Error string
}
```

#### `FfiNoneResult`

```pen
type FfiNoneResult {
  None none
  Error string
}
```

#### `FfiReadDirectoryResult`

```pen
type FfiReadDirectoryResult {
  Paths array'Array
  Error string
}
```

#### `FfiMetadataResult`

```pen
type FfiMetadataResult {
  Metadata Metadata
  Error string
}
```

#### `FfiTcpListenerResult`

```pen
type FfiTcpListenerResult {
  Listener Tcp'Listener
  Error string
}
```

#### `FfiTcpStreamResult`

```pen
type FfiTcpStreamResult {
  Stream Tcp'Stream
  Error string
}
```

#### `FfiTcpAcceptedStreamResult`

```pen
type FfiTcpAcceptedStreamResult {
  Stream Tcp'AcceptedStream
  Error string
}
```

#### `FfiUdpSocketResult`

```pen
type FfiUdpSocketResult {
  Socket Udp'Socket
  Error string
}
```

#### `FfiUdpDatagramResult`

```pen
type FfiUdpDatagramResult {
  Datagram Udp'Datagram
  Error string
}
```

### Functions

#### `UnsafeNew`

```pen
\() Context
```

#### `Inner`

```pen
\(ctx Context) InnerContext
```

## `Os'Directory` module

### Types

No types are defined.

### Functions

#### `Read`

```pen
\(ctx Context, path string) [string] | error
```

#### `Create`

```pen
\(ctx Context, path string) none | error
```

#### `Remove`

```pen
\(ctx Context, path string) none | error
```

## `Os'Environment` module

### Types

No types are defined.

### Functions

#### `Arguments`

```pen
\(ctx Context) [string]
```

#### `Variable`

```pen
\(ctx Context, name string) string | error
```

## `Os'File` module

### Types

#### `File`

```pen
type File {
  ...
}
```

### Functions

#### `StdIn`

```pen
\() File
```

#### `StdOut`

```pen
\() File
```

#### `StdErr`

```pen
\() File
```

#### `OpenWithOptions`

```pen
\(ctx Context, path string, opt OpenOptions) File | error
```

#### `Open`

```pen
\(ctx Context, path string) File | error
```

#### `Read`

```pen
\(ctx Context, file File) string | error
```

#### `ReadLimit`

```pen
\(ctx Context, file File, limit number) string | error
```

#### `Write`

```pen
\(ctx Context, file File, data string) number | error
```

#### `Copy`

```pen
\(ctx Context, src string, dest string) none | error
```

#### `Move`

```pen
\(ctx Context, src string, dest string) none | error
```

#### `Remove`

```pen
\(ctx Context, path string) none | error
```

#### `Metadata`

```pen
\(ctx Context, path string) Metadata | error
```

## `Os'File'Metadata` module

### Types

#### `Metadata`

```pen
type Metadata {
  Size number
}
```

### Functions

No functions are defined.

## `Os'File'OpenOptions` module

### Types

#### `OpenOptions`

```pen
type OpenOptions {
  Append boolean
  Create boolean
  CreateNew boolean
  Read boolean
  Truncate boolean
  Write boolean
}
```

### Functions

#### `Default`

```pen
\() OpenOptions
```

## `Os'Process` module

### Types

No types are defined.

### Functions

#### `Exit`

```pen
\(ctx Context, code number) none
```

## `Os'Random` module

### Types

No types are defined.

### Functions

#### `Number`

```pen
\(ctx Context) number
```

## `Os'Tcp` module

### Types

No types are defined.

### Functions

#### `Bind`

```pen
\(ctx Context, address string) Listener | error
```

#### `Connect`

```pen
\(ctx Context, address string) Stream | error
```

#### `Accept`

```pen
\(ctx Context, l Listener) AcceptedStream | error
```

#### `Receive`

```pen
\(ctx Context, s Stream, limit number) string | error
```

#### `Send`

```pen
\(ctx Context, s Stream, data string) number | error
```

## `Os'Tcp'AcceptedStream` module

### Types

#### `AcceptedStream`

```pen
type AcceptedStream {
  Stream Stream
  Address string
}
```

### Functions

No functions are defined.

## `Os'Tcp'Listener` module

### Types

#### `Listener`

```pen
type Listener {
  ...
}
```

### Functions

No functions are defined.

## `Os'Tcp'Stream` module

### Types

#### `Stream`

```pen
type Stream {
  ...
}
```

### Functions

No functions are defined.

## `Os'Time` module

### Types

No types are defined.

### Functions

#### `Now`

```pen
\(ctx Context) number
```

#### `Sleep`

```pen
\(ctx Context, milliseconds number) none
```

## `Os'Udp` module

### Types

No types are defined.

### Functions

#### `Bind`

```pen
\(ctx Context, address string) Socket | error
```

#### `Connect`

```pen
\(ctx Context, s Socket, address string) none | error
```

#### `Receive`

```pen
\(ctx Context, s Socket) string | error
```

#### `ReceiveFrom`

```pen
\(ctx Context, s Socket) Datagram | error
```

#### `Send`

```pen
\(ctx Context, s Socket, data string) number | error
```

#### `SendTo`

```pen
\(ctx Context, s Socket, data string, address string) number | error
```

## `Os'Udp'Datagram` module

### Types

#### `Datagram`

```pen
type Datagram {
  Data string
  Address string
}
```

### Functions

No functions are defined.

## `Os'Udp'Socket` module

### Types

#### `Socket`

```pen
type Socket {
  ...
}
```

### Functions

No functions are defined.
