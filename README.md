viax cli
------------

A CLI tool to manage state of viax cloud. It includes:
* det/deploy/delete functions
* det/deploy/delete integrations

To see what is available at the moment run:
```
$ viax help

Usage: viax [ENV] <COMMAND>

Commands:
  fn    Functions management commands
  int   Inegrations management commands
  help  Print this message or the help of the given subcommand(s)

Arguments:
  [ENV]

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Roadmap:
* generate a function project
* generate an integration project
* manage viax configurations
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

Note: if you are on macos, after you download a binary run `xattr -c viax` or allow execution through "Settings -> Privacy & Security".

# Examples

Get a function:
```
$ viax fn get send-email-task
NAME                           READY DEPLOY_STATUS        VERSION  REVISION
send-email-task                True  Ready                1.0.0    send-email-task-00005
```

To deploy a function you a to zip it beforehand (there is a plan to zip on the fly in future):
```
$ cd my-fun
$ zip -r int.zip .
$ viax fn deploy fn.zip
Enqueued deployment:
uid: 679342d0-6b1d-44a4-aa02-47a485cd1128, deploy status: EnqueuedDeploying
Note: last deployed function will be working until new function is deployed. Previously deployed:
ready: Unknown, revision: Unknown
```
