module RequestHelper
  def post(endpoint, body)
    Faraday.post(
      "http://localhost:6353#{endpoint}",
      body.to_json,
      { 'Content-Type' => 'application/json' }
    )
  end
end
