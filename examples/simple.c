#include<stdlib.h>
#include<stdio.h>
#include<string.h>
#include<getopt.h>


static void usage(const char *progname) {
	printf("usage: %s [options] [TODO ...]\n%s", progname,
	       "  -h  --help\n"
	       "        print this usage and exit\n"
	       "  -b  --block-size  --bs\n"
	       "        this is help text for block_size\n"
	       "  -q  --max_warp\n"
	       "  -n  --cores\n"
	       );
}

void parse_args(int argc, char **argv, char* *out_file, char* *in_file, int *block_size, int *max_warp, char* *username) {
	int block_size__isset = 0;
	int username__isset = 0;
	static struct option longopts[] = {
		{"block-size", required_argument, 0, 98},
		{"max_warp", no_argument, 0, 113},
		{"cores", required_argument, 0, 110},
		{"help", 0, 0, 'h'},
		{0, 0, 0, 0}
	};
	int ch;
	while ((ch = getopt_long(argc, argv, "b:qn:h", longopts, NULL)) != -1) {
		switch (ch) {
		case 98:
			*block_size = atoi(optarg);
			block_size__isset = 1;
			break;
		case 113:
			*max_warp = 1;
			break;
		case 110:
			*username = optarg;
			username__isset = 1;
			break;
		case 0:
			break;
		case 'h':
		default:
			usage(argv[0]);
			exit(1);
		}
	}
	if (!block_size__isset) {
		usage(argv[0]);
		exit(1);
	}
	if (!username__isset) {
		*username = "John Smith";
	}

	while (optind < argc) {
		/* TODO: positional loop */
		optind++;
	}
}

int main(int argc, char **argv) {
	char* out_file;
	char* in_file;
	int block_size;
	int max_warp;
	char* username;

	parse_args(argc, argv, &out_file, &in_file, &block_size, &max_warp, &username);

	/* TODO: call your code here */
}
