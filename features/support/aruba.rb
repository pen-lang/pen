require 'aruba/cucumber'

Aruba.configure do |config|
  config.exit_timeout = 60
  config.home_directory = ENV['HOME']
end
