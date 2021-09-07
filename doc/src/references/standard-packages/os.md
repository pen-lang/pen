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

## `Os` module

```pen
import System'Os
```

### Types

#### `Context`

It is a context of the operating system. It must be passed to functions in this module.

```pen
type Context { ... }
```

#### `File`

It is a file.

```pen
type File { ... }
```

#### `OpenFileOptions`

It is options to open a file. Its flags are described below.

- `Append` allows appending data to the file.
- `Create` creates a new file if the file doesn't exist or opens it otherwise.
- `CreateNew` creates a new file. If the file already exists, it fails to open the file.
- `Read` allows reading data from the file.
- `Truncate` truncates the file first to 0-byte size.
- `Write` allows writing data to the file.

```pen
type OpenFileOptions {
  Append boolean
  Create boolean
  CreateNew boolean
  Read boolean
  Truncate boolean
  Write boolean
}
```

### Functions

#### `Arguments`

It gets command line arguments.

```pen
\(ctx Context) [string]
```

#### `EnvironmentVariable`

It gets an environment variable of a given name. It returns an error if the variable is undefined.

```pen
\(ctx Context, name string) string | error
```

#### `StdIn`

It gets a file for standard input.

```pen
\() File
```

#### `StdOut`

It gets a file for standard output.

```pen
\() File
```

#### `StdErr`

It gets a file for standard error.

```pen
\() File
```

#### `DefaultOpenFileOptions`

It gets default open file options where all flags are set `false`.

```pen
\() OpenFileOptions
```

#### `OpenFileWithOptions`

It opens a file with options.

```pen
\(ctx Context, path string, opts OpenFileOptions) File | error
```

#### `OpenFile`

It opens a file for read-only.

```pen
\(ctx Context, path string) File | error
```

#### `ReadFile`

It reads all data from a file.

```pen
\(ctx Context, file File) string | error
```

#### `WriteFile`

It writes data to a file and returns a number of bytes written to the file.

```pen
\(ctx Context, file File, data string) number | error
```

#### `CopyFile`

It copies a file to another path.

```pen
\(ctx Context, src string, dest string) none | error
```

#### `RemoveFile`

It removes a file at a path.

```pen
\(ctx Context, path string) none | error
```

#### `ReadDirectory`

It reads a directory and returns a list of files and directories inside.

```pen
\(ctx Context, path string) [string] | error
```

#### `CreateDirectory`

It creates a directory.

```pen
\(ctx Context, path string) none | error
```

#### `RemoveDirectory`

It removes a directory.

```pen
\(ctx Context, path string) none | error
```
