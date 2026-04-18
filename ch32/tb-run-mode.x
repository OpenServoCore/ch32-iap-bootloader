/* Magic word at end of RAM. memory.x reserves these 4 bytes. */
__tb_run_mode = ORIGIN(RAM) + LENGTH(RAM);
