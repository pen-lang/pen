require 'aruba/cucumber'

Aruba.configure do |config|
  config.exit_timeout = 120
  config.home_directory = ENV['HOME']
end
