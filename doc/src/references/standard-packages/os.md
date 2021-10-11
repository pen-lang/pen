# OS

This package provides an interface for operating systems. Currently, it supports Linux, macOS and [WASI](https://wasi.dev).

## Install

```json
{
  "dependencies": {
    "System": "pen:///lib/os"
  }
}
```

## `Context` module

```pen
import System'Context
```

### Types

#### `Context`

It is a context of an operating system.

```pen
type Context { ... }
```

#### `Metadata`

It is file metadata.

```pen
type Metadata {
  Size number
}
```

## `Environment` module

```pen
import System'Environment
```

### Functions

#### `Arguments`

It gets command line arguments.

```pen
\(ctx Context) [string]
```

#### `Variable`

It gets an environment variable of a given name. It returns an error if the variable is undefined.

```pen
\(ctx Context, name string) string | error
```

## `File` module

```pen
import System'File
```

### Types

#### `File`

It is a file.

```pen
type File { ... }
```

### Functions

#### `StdIn`

It returns a file for standard input.

```pen
\() File
```

#### `StdOut`

It returns a file for standard output.

```pen
\() File
```

#### `StdErr`

It returns a file for standard error.

```pen
\() File
```

#### `Open`

It opens a file to read.

```pen
\(ctx Context, path string) File | error
```

#### `OpenWithOptions`

It opens a file with options.

```pen
\(ctx Context, path string, opt OpenOptions) File | error
```

#### `Read`

It reads all data from a file.

```pen
\(ctx Context, file File) string | error
```

#### `ReadLimit`

It reads data from a file until a limit in bytes.

```pen
\(ctx Context, file File, limit number) string | error
```

#### `Write`

It writes data to a file and returns a number of bytes written to the file.

```pen
\(ctx Context, file File, data string) number | error
```

#### `Copy`

It copies a file to another path.

```pen
\(ctx Context, src string, dest string) none | error
```

#### `Remove`

It removes a file at a path.

```pen
\(ctx Context, path string) none | error
```

## `File'OpenOptions` module

```pen
import System'File'OpenOptions
```

### Types

#### `OpenOptions`

It is options to open a file. Its flags are described below.

- `Append` allows appending data to the file.
- `Create` creates a new file if the file doesn't exist or opens it otherwise.
- `CreateNew` creates a new file. If the file already exists, it emits an error.
- `Read` allows reading data from the file.
- `Truncate` truncates the file to zero byte.
- `Write` allows writing data to the file.

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

It gets default options where all flags are set `false`.

```pen
\() OpenOptions
```

## `Directory` module

```pen
import System'Directory
```

### Functions

#### `Read`

It reads a directory and returns a list of files and directories inside.

```pen
\(ctx Context, path string) [string] | error
```

#### `Create`

It creates a directory.

```pen
\(ctx Context, path string) none | error
```

#### `Remove`

It removes a directory.

```pen
\(ctx Context, path string) none | error
```

#### `Metadata`

It reads file metadata.

```pen
\(ctx Context, path string) Metadata | error
```
