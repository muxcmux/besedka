require 'openssl'
require 'base64'
require 'json'

module Utils
  def encode(data)
    Base64.strict_encode64(data)
  end

  def sign(data, secret)
    secret_bytes = Base64.strict_decode64(secret)
    digest = OpenSSL::Digest.new('sha256')
    signature = Base64.strict_encode64(OpenSSL::HMAC.digest(digest, secret_bytes, data))

    [encode(data), signature]
  end
end
