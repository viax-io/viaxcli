viax cli
------------

A CLI tool to manage state of viax cloud. It includes:
* deploy functions
* deploy integrations

To see what is available at the moment run:
```
$ viax help

Usage: viax [ENV] [COMMAND]

Commands:
  deploy-int  Deploy an integration
  deploy-fn   Deploy a function
  help        Print this message or the help of the given subcommand(s)

Arguments:
  [ENV]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Roadmap:
* full management of funcitons and integrations: delete/deploy/getinfo
* generate a function project
* generate an integration project
* to be added

# Configuration

Configuration file is stored in a `<user_home>/.viax/config`. Example:
```
realm = "viax"
[default]
client_id = "demo-client"
client_secret = "a81c3b72-0c8f-4885-b888-9999fa123455"
auth_url = "https://auth.viax.demo.viax.io"
api_url = "https://api.viax.demo.viax.io/graphql"
[dev]
client_id = "demo-client"
client_secret = "a81c3b72-0c8f-4885-b888-9999fa123455"
```

Specify your realm at the top of file, e.g. `realm = "viax"`. Then in square brackats it is env specific configuration, required:
* client_id
* client_secret
Optional, if not specified it is built from a pattern:
* auth_url - pattern for a default one `https://auth.{realm}.{env}.viax.io`
* api_url patter for a default url `https://api.{realm}.{env}.viax.io/graphql`

