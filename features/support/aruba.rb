require 'aruba/cucumber'

Aruba.configure do |config|
  config.exit_timeout = 60
  config.home_directory = File.join(
    config.root_directory, 'tmp/home'
  )
end
