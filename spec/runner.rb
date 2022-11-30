require 'open3'

class Runner
  attr_accessor :wait_thread, :stopped

  def start(print_to_stdout: true)
    `touch test.sqlite && DATABASE_URL=sqlite://test.sqlite sqlx migrate run`

    self.stopped = false

    _, out, self.wait_thread = Open3.popen2e('target/debug/besedka s --db test.sqlite')

    Process.detach(wait_thread.pid)

    while (line = out.gets)
      break if line.match(/Listening on 0.0.0.0:6353/)
    end

    return unless print_to_stdout

    Thread.new do
      while (line = out.gets)
        puts line
      end
    end
  end

  def stop
    return if stopped

    self.stopped = true

    Process.kill('SIGINT', wait_thread.pid) unless wait_thread.nil?

    ['test.sqlite', 'test.sqlite-journal'].each do |f|
      File.delete(f) if File.exist?(f)
    end
  end
end
