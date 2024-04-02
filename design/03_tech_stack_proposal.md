# Proposal number 3: Tech stack and tools proposal

For the developpement of this tool, the following tech stack may be used

- **rust**: Developping a CLI in rust will provide both the safety required for
  a backup tool, a single executable easy to release and install for users, low 
  level access to files, but with easier syntax than c or c++. Backup are quite
  a sensitive operation that might benefit from the strong garantees brought by
  rust.

- **TOML**: (Tom's obvious minimal language) is a readable and expressive format 
  for representing configuration data, featuring a human-friendly syntax, support 
  for diverse data types and structures, as well as hierarchical organization. 
  It would therefore be a good way to represent both global configuration of 
  the tool and individual configurations of projects backup requirements

- **Async over threads**: The concurrency model of rust exposes mainly 2 ways to 
  perform actions in parallel. OS threads and async/await. OS threads are 
  expensive to create (that might be mitigated by thread pools, but it's more 
  complex). For IO intensive apps, the async/await model is more efficient, 
  allowing for a large number of IO operations in parallel (order of magnitude
  larger than the number of cores). For CPU bound application however, the 
  OS threads might worth it. In our case, most operations will be IO: walk the 
  filesystem for projects, read configuration files, send archives from drive
  to drive, ... The only CPU intensive tasks might be compression and encryption
  in the future.

- **rustfmt, clippy**: To guarantee consistant code style and enforce good 
  practices, rustfmt shall be used as the formatter of this project, and clippy
  as the linter.

