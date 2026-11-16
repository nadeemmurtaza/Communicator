#!/usr/bin/env bash
# Starts two backend-rs instances on one Redis and one scratch database, then
# checks the shared rate limit, per-identifier limit, lockout and input validation.
# Needs Postgres, Redis (REDIS_URL) and a built target/debug/backend-rs.
set -u
cd "$(dirname "$0")/.."
: "${REDIS_URL:?set REDIS_URL}"
PGADMIN=${PGADMIN:-postgresql://nid:nid@localhost:5432/postgres}
DB=comm_security_smoke
psql "$PGADMIN" -qc "drop database if exists $DB" -c "create database $DB" >/dev/null 2>&1
redis-cli -u "$REDIS_URL" flushall >/dev/null
export RUST_DATABASE_URL=${PGADMIN%/*}/$DB RL_AUTH_PER_MIN=10 RL_AUTH_PER_HOUR=60 RL_IP_PER_MIN=300 RL_IDENT_PER_10MIN=3 \
  RL_AUTHFAIL_PER_10MIN=5 RL_LOCKOUT_SECS=30 RL_BADREQ_PER_10MIN=1000
# One at a time: the first instance applies the migrations.
for p in 8101 8102; do
  (PORT=$p setsid ./target/debug/backend-rs >/tmp/comm-$p.log 2>&1 < /dev/null &)
  for _ in $(seq 1 120); do curl -sf localhost:$p/api/v1/health >/dev/null && break; sleep 1; done
done
PASS=0; FAIL=0
ok() { if [ "$2" = "$3" ]; then PASS=$((PASS+1)); echo "ok   $1"; else FAIL=$((FAIL+1)); echo "FAIL $1 (expected $3, got $2)"; fi; }
post() { curl -s -o /dev/null -w '%{http_code}' -X POST "localhost:$1$2" -H 'content-type: application/json' -d "$3"; }

ok "bad email rejected" "$(post 8101 /api/v1/auth/sign-up/start '{"email":"nope"}')" 422
ok "NUL byte rejected" "$(post 8101 /api/v1/auth/sign-up/start '{"email":"a@b.co","name":"x\u0000y"}')" 422
ok "HTML tag in a name rejected" "$(post 8101 /api/v1/auth/sign-up/start '{"email":"a@b.co","name":"<script>x</script>"}')" 422
ok "wrong content type rejected" "$(curl -s -o /dev/null -w '%{http_code}' -X POST localhost:8101/api/v1/auth/sign-up/start -H 'content-type: text/plain' -d '{"email":"a@b.co"}')" 415
ok "bad JSON rejected" "$(post 8101 /api/v1/auth/sign-up/start '{"email":')" 400
ok "path traversal rejected" "$(curl -s -o /dev/null -w '%{http_code}' 'localhost:8101/api/v1/..%2F..%2Fetc')" 400
ok "oversized body rejected" "$(python3 -I -c "print('{\"name\":\"' + 'a'*2200000 + '\"}')" | curl -s -o /dev/null -w '%{http_code}' -X POST localhost:8101/api/v1/auth/sign-up/start -H 'content-type: application/json' --data-binary @-)" 413
ok "webhook bodies are not field-validated" "$([ "$(post 8101 /api/v1/commerce/webhooks/shopify '{"name":"<b>x</b>","phone":"zzz"}')" != 422 ] && echo y)" y

redis-cli -u "$REDIS_URL" flushall >/dev/null
allowed=0; for i in $(seq 1 14); do c=$(post $((8101 + i % 2)) /api/v1/auth/sign-up/verify '{"x":1}'); [ "$c" != 429 ] && allowed=$((allowed+1)); done
ok "10 sign-in requests allowed across two instances, the rest refused" "$allowed" 10

redis-cli -u "$REDIS_URL" flushall >/dev/null
allowed=0; for i in $(seq 1 6); do c=$(post $((8101 + i % 2)) /api/v1/auth/sign-up/start '{"email":"same@person.co"}'); [ "$c" != 429 ] && allowed=$((allowed+1)); done
ok "per-identifier limit (3) holds across instances" "$allowed" 3
ok "a different identifier is unaffected" "$([ "$(post 8101 /api/v1/auth/sign-up/start '{"email":"other@person.co"}')" != 429 ] && echo y)" y

redis-cli -u "$REDIS_URL" flushall >/dev/null
for i in $(seq 1 7); do curl -s -o /dev/null -H 'authorization: Bearer nope' "localhost:$((8101 + i % 2))/api/v1/auth/me"; done
ok "repeated failed auth locks the address out on BOTH instances" "$(curl -s -o /dev/null -w '%{http_code}' localhost:8101/api/v1/auth/me)$(curl -s -o /dev/null -w '%{http_code}' localhost:8102/api/v1/auth/me)" 429429
ok "health stays reachable during a lockout" "$(curl -s -o /dev/null -w '%{http_code}' localhost:8101/api/v1/health)" 200
ok "security headers present" "$(curl -s -D - -o /dev/null localhost:8101/api/v1/health | grep -ci 'x-content-type-options')" 1

pkill -f "target/debug/backend-rs" 2>/dev/null; sleep 1
psql "$PGADMIN" -qc "drop database if exists $DB" >/dev/null 2>&1
echo; echo "$PASS passed, $FAIL failed"; [ "$FAIL" -eq 0 ]
