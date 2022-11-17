require "openssl"
require "base64"
require "json"

config = {
  comments_per_page: 100,
  # user: {
  #   id: "42",
  #   moderator: true
  # }
}.to_json

base64_secret = 'ITKV6bBqlncnBIB438SjwZdEn0TSGxQpjqm62PqTt+o='
secret_bytes = Base64.strict_decode64(base64_secret)
digest = OpenSSL::Digest.new('sha256')
signature = Base64.strict_encode64(OpenSSL::HMAC.digest(digest, secret_bytes, config))

puts config
puts Base64.strict_encode64(config)
puts "Signature: #{signature}"
