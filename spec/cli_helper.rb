module CliHelper
  def add_site(site, **kwargs)
    command('sites', 'add', site, **kwargs).lines.find { |l| l.match(/^secret:/) }.split(':').last.strip
  end

  def command(cmd, *args, **kwargs)
    opts = kwargs.keys.map do |k|
      "--#{k.to_s.gsub('_', '-')} #{kwargs[k]}"
    end

    `target/debug/besedka #{cmd} #{args.join(' ')} #{opts.join(' ')} --db test.sqlite`
  end
end
