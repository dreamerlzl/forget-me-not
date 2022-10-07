# overview
- a command-line task reminder sending desktop notifications for linux/macOS
  - relying on [notify-rust](https://github.com/hoodie/notify-rust) it's cross-platform out of box
- consisting of two executables, a client and a daemon
- tasks are stored as a file for persistence
  - configure it via env var `REMINDER_TASK_STORE`

# usage
## client
```bash
# help
fmn -h

# examples
# remind after 5 mins
fmn add "just a reminder" after 5m

# remind per hour
fmn add "hello world" per 1h

# remind me at 19:30 today (assuming it's in the future)
fmn add "foo bar" at 19:30

# show all reminder tasks
fmn show

# remove a task
fmn rm <task_id>
```

## daemon
- for linux, you would need to deploy it via `systemd` or `initd`
- for macOS, you would need to use launchd
- fmn-daemon uses udp for listening
  - configure the port to use via env var `REMINDER_DAEMON_ADDR` (localhost:8082 by default)
