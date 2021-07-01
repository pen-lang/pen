require 'aruba/cucumber'

Aruba.configure do |config|
  config.home_directory = ENV['HOME']
end
