/*
   Copyright 2019 Ian Boll

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

.section ".text.boot"

.global _boot_cores

_boot_cores:
    // read cpu id, stop slave cores
    mrs   x1, mpidr_el1
    and   x1, x1, #3       // mask only the bits containing the core number
    cbz   x1, 2f           // if zero, continue to boot the kernel
1:  wfe                    // stop the core
    b     1b               // if the stop is ended, restop

2:  ldr   x1, =_boot_cores // set this "function" as the top of the stack
    mov   sp, x1
    bl    reset            // call into the rust code, shouldn't return
    b     1b               // stop this core as a failsafe
