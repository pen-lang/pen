require 'aruba/cucumber'

Aruba.configure do |config|
  config.exit_timeout = 60
  config.home_directory = File.join(
    config.root_directory, config.working_directory
  )
end
