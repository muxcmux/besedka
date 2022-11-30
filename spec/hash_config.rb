require "openssl"
require "base64"
require "json"

user = {
  name: "Signed user",
  moderator: false
}.to_json

base64_secret = ARGV[0]
secret_bytes = Base64.strict_decode64(base64_secret)
digest = OpenSSL::Digest.new('sha256')
signature = Base64.strict_encode64(OpenSSL::HMAC.digest(digest, secret_bytes, user))

puts user
puts Base64.strict_encode64(user)
puts "Signature: #{signature}"
