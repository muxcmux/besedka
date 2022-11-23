require "openssl"
require "base64"
require "json"

user = {
  name: "Antoni",
  moderator: false,
  avatar: "data:image/svg+xml;base64,PD94bWwgdmVyc2lvbj0iMS4wIiBlbmNvZGluZz0iaXNvLTg4NTktMSI/Pg0KPCEtLSBHZW5lcmF0b3I6IEFkb2JlIElsbHVzdHJhdG9yIDE5LjAuMCwgU1ZHIEV4cG9ydCBQbHVnLUluIC4gU1ZHIFZlcnNpb246IDYuMDAgQnVpbGQgMCkgIC0tPg0KPHN2ZyB2ZXJzaW9uPSIxLjEiIGlkPSJMYXllcl8xIiB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHhtbG5zOnhsaW5rPSJodHRwOi8vd3d3LnczLm9yZy8xOTk5L3hsaW5rIiB4PSIwcHgiIHk9IjBweCINCgkgdmlld0JveD0iMCAwIDQ1OSA0NTkiIHN0eWxlPSJlbmFibGUtYmFja2dyb3VuZDpuZXcgMCAwIDQ1OSA0NTk7IiB4bWw6c3BhY2U9InByZXNlcnZlIj4NCjxnPg0KCTxnPg0KCQk8cGF0aCBkPSJNMjI5LjUsMEMxMDIuNTMsMCwwLDEwMi44NDUsMCwyMjkuNUMwLDM1Ni4zMDEsMTAyLjcxOSw0NTksMjI5LjUsNDU5QzM1Ni44NTEsNDU5LDQ1OSwzNTUuODE1LDQ1OSwyMjkuNQ0KCQkJQzQ1OSwxMDIuNTQ3LDM1Ni4wNzksMCwyMjkuNSwweiBNMzQ3LjYwMSwzNjQuNjdDMzE0Ljg4NywzOTMuMzM4LDI3My40LDQwOSwyMjkuNSw0MDljLTQzLjg5MiwwLTg1LjM3Mi0xNS42NTctMTE4LjA4My00NC4zMTQNCgkJCWMtNC40MjUtMy44NzYtNi40MjUtOS44MzQtNS4yNDUtMTUuNTk3YzExLjMtNTUuMTk1LDQ2LjQ1Ny05OC43MjUsOTEuMjA5LTExMy4wNDdDMTc0LjAyOCwyMjIuMjE4LDE1OCwxOTMuODE3LDE1OCwxNjENCgkJCWMwLTQ2LjM5MiwzMi4wMTItODQsNzEuNS04NGMzOS40ODgsMCw3MS41LDM3LjYwOCw3MS41LDg0YzAsMzIuODEyLTE2LjAyMyw2MS4yMDktMzkuMzY5LDc1LjAzNQ0KCQkJYzQ0Ljc1MSwxNC4zMTksNzkuOTA5LDU3Ljg0OCw5MS4yMTMsMTEzLjAzOEMzNTQuMDIzLDM1NC44MjgsMzUyLjAxOSwzNjAuNzk4LDM0Ny42MDEsMzY0LjY3eiIvPg0KCTwvZz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjxnPg0KPC9nPg0KPGc+DQo8L2c+DQo8Zz4NCjwvZz4NCjwvc3ZnPg0K"
}.to_json

base64_secret = 'dfRucT7/a0IkrhiPw3z80jbyc1PkVKJMLC5eXXZ2Hs6TXk9C4YkoR/vmB7trbpQz'
secret_bytes = Base64.strict_decode64(base64_secret)
digest = OpenSSL::Digest.new('sha256')
signature = Base64.strict_encode64(OpenSSL::HMAC.digest(digest, secret_bytes, user))

puts user
puts Base64.strict_encode64(user)
puts "Signature: #{signature}"