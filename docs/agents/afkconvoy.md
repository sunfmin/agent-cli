# afkconvoy config

How the afkconvoy fleet works this repo's backlog: issues labelled
`ready-for-agent`, worked by the team
[sunfmin/product-team](https://github.com/sunfmin/product-team), landed on
`main` behind `./gate.sh`, the same gate a person runs.

```yaml
team: sunfmin/product-team
base_branch: "main"
gate_command: ./gate.sh
gate_seconds: 1800
ready_label: ready-for-agent
concurrency: 3
```
