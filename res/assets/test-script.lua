function setup()
	log.info("Hello from the lua script!")
end

function update()
	if deltatime_ms() > 10.0 then
		log.info("Frame spike detected! "..deltatime_ms().."ms")
	end
end