# http static server
Simple HTTP static server in Rust

This is more an experiment made to learn Rust than a real HTTP static server, however I'll try to improve it over time.

The program can be launched as:
* <code>http_static_server</code>: this way the server is listening on the predefined port 8080 providing the current directory as content.
* <code>http_static_server &lt;options&gt;</code>: 
  * this way the server can be launched specifying a different port and/or a different working directory;
  * <code>--help</code>: show all the supported options.
