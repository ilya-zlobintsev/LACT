# Description

The LACT Daemon exposes a JSON API over a unix socket or TCP, available on `/var/run/lactd.sock` or an arbitrary TCP port. You can configure who has access to the unix socket in `/etc/lact/config.yaml` in the `daemon.admin_groups` field. The TCP listener is disabled by default for security reasons, see [this README section](./README.md#remote-management) for how to enable it.

The API expects newline-separated JSON objects, and returns a JSON object for every request.

The general format of requests looks like:
```
{"command": "command_name", "args": {}}
```
Note that the type of `args` depends on the specific request, and may be ommited in some cases.

The response looks like this:
```
{"status": "ok|error", "data": {}}
```
Same as `args` in requests, `data` can be of a different type and may not be present depending on the specific request.

You can try sending commands to socket interactively with `ncat`:
```
echo '{"command": "list_devices"}' | ncat -U /run/lactd.sock
```
Example response:
```
{"status":"ok","data":[{"id":"1002:687F-1043:0555-0000:0b:00.0","name":"Vega 10 XL/XT [Radeon RX Vega 56/64]"}]}
```

Here's an example of calling the API with arguments to change a profile:
```
echo '{"command": "set_profile", "args": {"name":"name-of-the-profile"}}' | ncat -U /run/lactd.sock
```
In this code, `name-of-the-profile` should be replaced with the name of a profile that you've already created in LACT.


# Commands

For the full list of available commands and responses, you can look at the source code of the schema: [requests](lact-schema/src/request.rs), [the basic response structure](lact-schema/src/response.rs) and [all possible types](lact-schema/src/lib.rs).

It should also be fairly easy to figure out the API by trial and error, as the error message are quite verbose:

```
echo '{"command": "test"}' | ncat -U /run/lactd.sock

{"status":"error","data":"Failed to deserialize request: unknown variant `test`, expected one of `ping`, `list_devices`, `system_info`, `device_info`, `device_stats`, `device_clocks_info`, `set_fan_control`, `set_power_cap`, `set_performance_level`, `set_clocks_value` at line 1 column 18"}
```

# Rust

If you want to connect to the socket from a Rust program, you can simply import either the `lact-client` or `lact-schema` (if you want to write a custom client) crates from this repository.