# Caminho relativo: cmake/NeoCADTargets.cmake

function(neocad_add_pnpm_target target_name)
	add_custom_target(
		${target_name}
		COMMAND ${PNPM_EXECUTABLE} ${ARGN}
		WORKING_DIRECTORY ${NEOCAD_ROOT_DIR}
		USES_TERMINAL
		VERBATIM
	)
endfunction()
