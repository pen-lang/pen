## Background

Simplicity enables efficient collaboration of developers. [The Go programming language](https://golang.org) has been notably successful as it's been one of the most simple but practical programming languages ever made. That being said, [Go 2](https://go.dev/blog/go2-here-we-come) decided to compromise some complexity for its evolution, such as its [generics](https://github.com/golang/go/issues/43651) proposal.

On the other hand, Pen aims to be **even simpler by focusing only on application programming** as its target domain while adopting the same philosophy of simplicity. It pursues its minimal language design further after removing several features from Go like pointers, mutability, method syntax, global variables, circular references, etc.

Furthermore, although many programming languages have been solving problems of **programming** in history, few of them actually tackled ones of **software engineering**, where you also need to maintain and keep making changes to existing software continuously. Pen's approach to that is embracing battle-tested ideas in such field, such as [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html), into the language's principles and ecosystem. One of its most clear incarnations is [system injection](/advanced-features/system-injection.html).
