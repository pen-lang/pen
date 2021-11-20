require 'aruba/cucumber'

Aruba.configure do |config|
  config.exit_timeout = 120
  config.home_directory = File.join(
    config.root_directory, 'tmp/home'
  )
end
