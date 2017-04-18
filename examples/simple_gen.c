#include <stdio.h>
#include <string.h>
#include <stdlib.h>

// Example Usage: "./a.out --int_arg 2 --char_arg c 4"

void populate_args(int * int_arg, char * char_arg, int * np_arg, int argc, char ** argv){
  for(int i = 1; i < argc; i++){
    if(!strcmp(argv[i], "--int_arg")){
      *int_arg = atoi(argv[i+1]);
    }
    else if(!strcmp(argv[i], "--char_arg")){
      *(char_arg) = argv[i + 1][0];
    }
    else{
      *np_arg = atoi(argv[i]);
    }
  }
}


int main(int argc, char ** argv){
  int int_arg;
  char char_arg;
  int np_arg;

  populate_args(&int_arg, &char_arg, &np_arg, argc, argv);

  // Now we access stuff
  printf("int_arg is %d\n", int_arg);
  printf("char_arg is %c\n", char_arg);
  printf("np_arg is %d\n", np_arg);
}
