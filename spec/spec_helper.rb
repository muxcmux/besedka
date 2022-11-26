require 'rubygems'
require 'bundler'

ENV['BUNDLE_GEMFILE'] ||= File.expand_path('../Gemfile', __dir__)
require 'bundler/setup' if File.exist?(ENV['BUNDLE_GEMFILE'])
Bundler.require(:default)

RSpec.configure do |config|
  require 'cli_helper'
  require 'request_helper'
  require 'utils'

  config.include RequestHelper
  config.include CliHelper
  config.include Utils

  config.expect_with :rspec do |expectations|
    expectations.include_chain_clauses_in_custom_matcher_descriptions = true
  end

  config.mock_with :rspec do |mocks|
    mocks.verify_partial_doubles = true
  end

  config.shared_context_metadata_behavior     = :apply_to_host_groups
  config.example_status_persistence_file_path = 'spec/examples.txt'
  config.warnings                             = true
  config.default_formatter                    = 'doc' if config.files_to_run.one?
  config.profile_examples                     = 10
  config.order                                = :random
  config.disable_monkey_patching!

  Kernel.srand config.seed

  config.before(:suite) do
    expect(system('cargo', 'build')).to be(true), 'Failed to build Besedka'

    require 'open3'
    _, out, $server_wait_thread = Open3.popen2e('target/debug/besedka s --db test.sqlite')

    Process.detach($server_wait_thread.pid)

    while line = out.gets
     break if line.match(/Listening on 0.0.0.0:6353/)
    end
  end

  config.after(:suite) do
    Process.kill('SIGINT', $server_wait_thread.pid)
  end

  config.before(:each) do
    `touch test.sqlite && DATABASE_URL=sqlite://test.sqlite sqlx migrate run`
  end

  config.after(:each) do
    ['test.sqlite', 'test.sqlite-journal'].each do |f|
      File.delete(f) if File.exist?(f)
    end
  end
end
