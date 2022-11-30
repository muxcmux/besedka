require 'rubygems'
require 'bundler'

ENV['BUNDLE_GEMFILE'] ||= File.expand_path('../Gemfile', __dir__)
require 'bundler/setup' if File.exist?(ENV['BUNDLE_GEMFILE'])
Bundler.require(:default)

require 'runner'
runner = Runner.new

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
  config.order                                = :random
  config.disable_monkey_patching!

  Kernel.srand config.seed

  config.before(:suite) do
    expect(system('cargo', 'build')).to be(true), 'Failed to build Besedka'
  end

  config.after(:suite) do
    runner.stop
  end

  config.before(:example) do
    runner.start print_to_stdout: false
  end

  config.after(:example) do
    runner.stop
  end
end
