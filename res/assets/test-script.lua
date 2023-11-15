function setup()
	log.info("Hello from the lua script!")
end

function update()
	if time.deltatime_ms() > 10.0 then
		log.info("Frame spike detected! "..time.deltatime_ms().."ms")
	end

	if (time.deltatime_ms() - (time.deltatime() * 1000.0)) > 1e-16 then
		log.error("Deltatime discreptency!")
	end
end