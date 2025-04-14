# Description

The LACT Daemon exposes a JSON API over a unix socket or TCP, available on `/run/lactd.sock` or an arbitrary TCP port. You can configure who has access to the unix socket in `/etc/lact/config.yaml` in the `daemon.admin_group` or `daemon.admin_user` field. The TCP listener is disabled by default for security reasons, see [this README section](../README.md#remote-management) for how to enable it.

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

The primary API commands for GPU configuration are `get_gpu_config` and `set_gpu_config`. It uses the same configuration format as described in [CONFIG.md](./CONFIG.md), except as JSON instead of YAML.

To change a setting through the API, you should fetch the current config, modify it, and then set it.
Additionally, each config change must be confirmed, or it will be automatically reverted after a short time period (5 seconds by default). This is needed to avoid saving settings that instantly crash a system.

**Deprecated commands**: there is a number of `set_` commands in the API that change individual settings. They are still functional for backwards compatibility purposes, but the `get_gpu_config`/`set_gpu_config` commands should be preferred instead.

Example: how to set the power limit through the API:

1. Get GPU id
```
> echo '{"command": "list_devices"}' | nc -U /run/lactd.sock 
{"status":"ok","data":[{"id":"10DE:2704-1462:5110-0000:09:00.0","name":"AD103 [GeForce RTX 4080]"}]}
```

2. Get current config
```
> echo '{"command": "get_gpu_config", "args": {"id": "10DE:2704-1462:5110-0000:09:00.0"}}' | nc -U /run/lactd.sock 
{"status":"ok","data":{"fan_control_enabled":false,"fan_control_settings":{"mode":"static","static_speed":1.0,"temperature_key":"edge","interval_ms":500,"curve":{"40":0.3,"50":0.35,"60":0.5,"70":0.75,"80":1.0},"spindown_delay_ms":2856,"change_threshold":2},"power_cap":320.0}}
```

3. Set a new config

The `power_cap` field has been changed from the previous config
```
> echo '{"command": "set_gpu_config", "args": {"id": "10DE:2704-1462:5110-0000:09:00.0", "config": {"fan_control_enabled":false,"fan_control_settings":{"mode":"static","static_speed":1.0,"temperature_key":"edge","interval_ms":500,"curve":{"40":0.3,"50":0.35,"60":0.5,"70":0.75,"80":1.0},"spindown_delay_ms":2856,"change_threshold":2},"power_cap":340.0}}}' | nc -U /run/lactd.sock
{"status":"ok","data":5}
```

4. Confirm new config
```
> echo '{"command": "confirm_pending_config", "args": {"command": "confirm"}}' | nc -U /run/lactd.sock
{"status":"ok","data":null}
```

For the full list of available commands and responses, you can look at the source code of the schema: [requests](lact-schema/src/request.rs), [the basic response structure](lact-schema/src/response.rs) and [all possible types](lact-schema/src/lib.rs).

It should also be fairly easy to figure out the API by trial and error, as the error message are quite verbose:

```
echo '{"command": "test"}' | ncat -U /run/lactd.sock

{"status":"error","data":"Failed to deserialize request: unknown variant `test`, expected one of `ping`, `list_devices`, `system_info`, `device_info`, `device_stats`, `device_clocks_info`, `set_fan_control`, `set_power_cap`, `set_performance_level`, `set_clocks_value` at line 1 column 18"}
```

# Rust

If you want to connect to the socket from a Rust program, you can simply import either the `lact-client` or `lact-schema` (if you want to write a custom client) crates from this repository.