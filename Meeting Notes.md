**9/28 Team Meeting Notes**

1. Research  
   1. look up similar information  
   2. compare with github repos  
   3. research projects need a lot of upfront info searched  
2. basic emulator? emulator \+ compiler? just a compiler without a CPU?  
3. Steps  
   1. subset of instructions from architecture  
      1. test out basic implementation  
      2. write in high-level code how you would execute an instruction (MOV, etc)  
      3. How much of the backend do you show in the frontend? emulation? compilation? LEDs?  
4. Languages  
   1. Hardware Description Language  
   2. use any

---

**10/2 Standup Meeting Notes**

1. Introductions  
   1. **Diego (C1)**: 4th year CS, strengths in full-stack development (frontend in React/Java and backend in AWS/Terraform) but preference in frontend work  
   2. **Ali (C1)**: 4th year CS, strengths in full-stack development but primarily backend and 400 level classes  
   3. **Avinash (C2)**: 6th year CS second degree, worked in the industry at Deloitte, strengths in full-stack development  
   4. **Sachin (C2)**: 4th year CS, strengths in project management and experience in internships, for programming stronger in backend/Azure  
   5. **Raveena (C1)**: 4th year CS, strengths in full-stack development and applications in JS/TS/Angular and Rust and 400 level classes  
   6. **Josh (C2)**: strengths in project management, programming experience from class  
   7. **Miranda (C1)**: 4th year CS, strengths in backend and project management and documentation  
   8. **Yuwei (C1)**: last year, strengths in frontend development  
2. Sprint Planning  
   1. delegated assignments for tasks in [Sprint Planning](https://docs.google.com/spreadsheets/d/1uwnrQXpdvr2GGtGOoDCePCjmKtcpdD3-G58972Alp_E/edit?gid=0#gid=0)  
   2. discussed research responsibilities  
3. Information  
   1. collected emails and github usernames to share Taiga board and Github repo  
   2. discussed recurring standup meetings and due dates

- x86 instruction set  
  - research how code is run in emulators  
- testing against existing emulator as benchmark w sample x86 code  
  - write algorithm for example instructions (parsing, performance, result)  
    - and translate to high level code  
    - flowchart  
  - note any additional details that may be necessary in the emulator

- adobe sign any assignments

---

**10/9 Standup Meeting Notes**

1. Share Taiga board and progress made  
2. Add results of tasks to Taiga board  
3. Update status of old Taiga board tasks  
   1. Closed out Sprint 1  
   2. Updated Sprint 2 with current tasks  
   3. Added evidence of completion to old tasks

---

**10/10 Team Meeting Notes**

1. **Avinash**: Finishing up the algorithm implementation task. Still needs to be tested.  
2. **Ali**: Found a comprehensive list of x86 instructions and examples of code online teaching how to write in assembly.  
3. **Diego**: test against different assemblers for assembly x86. Using online emulators that Raveena found. Specifically dissecting different emulators and seeing what some pain points are and finding ways to simplify the learning process.  
4. **Sachin**: Set up the git repository with React.   
5. **Veda:** Researching WASM sandboxing. Dependency on design UI WASM API Contract  
6. **Josh**: doing mock ui but needs to be assigned to the taiga project board  
7. **Miranda**: Researching visualizations libraries for our project. Still working on it but found a few libraries.  
8. **Raveena**: Looking at how to translate assembly to high level code. [8086 Emulator](https://yjdoc2.github.io/8086-emulator-web/) also moving everything from sprint 1 to sprint 2 or closing on the taiga board.  
9. **David**: Taiga board task needs to be updated. Made the threat model document.

**Instructor’s notes:**

* Good progress. Tried to address most comments made by Dr. Indela.  
* **Important:** Missing participants and need to bolster coordination. We need to have everyone involved. If this becomes an issue we need to inform Dr. Indela and Dr. Swathi  
* Good job on presentations. Try to rotate presentation leads for meetings.

---

**10/17 Standup Meeting Notes**

1. Share Taiga board and review Sprint 2 closed tasks and Sprint 3 in progress tasks  
   1. Diego: more research on existing emulators, Github Actions pipeline framework  
   2. Sachin: created the Github repository and deployed template frontend/backend files, debugged some build issues  
   3. Avinash: created Docker file and compose file, startup/shutdown commands  
      1. will possibly move docker files into folder  
   4. Miranda: Research Sandboxing, added notes to Taiga board and Master Document  
2. Further discussion on Emulator to HLC  
3. Todos  
   1. implementation of a few instructions in the backend (ADD, etc.)  
      1. start with zybooks MIPS solution  
      2. write corresponding x86 syntax from ^  
      3. one on ADD, one on SUBTRACT, etc. (unsigned, or twos complement by Friday) (everyone should take an instruction on)  
      4. narrow down set of instructions  
   2. flowchart that is detailed for an instruction  
   3. access zybooks and zylabs  
   4. add memory and breakpoints to mockup (LEDs later)  
4. How to split up work  
   1. Frontend  
   2. Backend  
      1. Git setup  
      2. Logic  
         1. Pseudocode  
         2. Implementation