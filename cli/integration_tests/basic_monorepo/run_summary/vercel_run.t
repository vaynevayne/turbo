# Setup
  $ . ${TESTDIR}/../../setup.sh
  $ . ${TESTDIR}/../setup.sh $(pwd)

# Kill what's running on port 8000 first, but also return 0 exit code if nothing is running on 8000
  $ PID=$(lsof -t -i:8000 2>/dev/null) && [[ -n $PID ]] && kill $PID || true
    
# Add a token into the directory so turborepo will think we are "linked".
  $ export XDG_CONFIG_HOME="${TESTDIR}"
  $ rm -rf "$TESTDIR/turborepo" || true
  $ mkdir -p "$TESTDIR/turborepo"
  $ echo "{\"token\": \"sometoken\"}" > $TESTDIR/turborepo/config.json

# Add a .turbo/config.json so there's a linked team for the monorepo
  $ rm -rf .turbo || true
  $ mkdir .turbo
  $ echo "{\"teamid\": \"a-team\"}" > .turbo/config.json

# Start mock server. Note if anything fails in the test run after this,
# the cleanup step won't run at the end so we have to be careful
# send stdout and stderr to /dev/null and also background the server
  $ python3 "${TESTDIR}/mock-api.py" --port 8000 &

# Run turbo
  $ TURBO_API=http://localhost:8000 TURBO_RUN_SUMMARY=true ${TURBO} run build --experimental-space-id=myspace

# Expect 3 POST requests
  $ ls post-*.json
  post-0.json
  post-1.json
  post-2.json

# And a PATCH request
  $ ls patch-*.json
  patch-0.json

# Check responses
## request-0.json should be the run URL
  $ cat post-0.json | jq '.requestUrl'
  "/v0/spaces/myspace/runs"

# Other post request should be to Tasks
  $ cat post-1.json | jq '.requestUrl'
  "/v0/spaces/myspace/runs/1/tasks"

  $ cat post-2.json | jq '.requestUrl'
  "/v0/spaces/myspace/runs/1/tasks"

# Spot check the first task POST
  $ cat post-1.json | jq '.requestBody | keys'
  [
    "cacheState",
    "command",
    "commandArguments",
    "dependencies",
    "dependents",
    "directory",
    "environmentVariables",
    "excludedOutputs",
    "execution",
    "expandedInputs",
    "expandedOutputs",
    "framework",
    "hash",
    "hashOfExternalDependencies",
    "logFile",
    "outputs",
    "package",
    "resolvedTaskDefinition",
    "task",
    "taskId"
  ]

# Patch request is pretty small so we can validate the URL and request payload together
  $ cat patch-0.json | jq
  {
    "requestUrl": "/v0/spaces/myspace/runs",
    "requestBody": {
      "Status": "completed"
    }
  }

# Kill mock server after.
  $ cat server.pid
  \d+ (re)

  $ kill -9 $(cat server.pid)
  $ rm server.pid

# Cleanup the config directory we made at the top of this test
$ rm -rf "$TESTDIR/turborepo"
