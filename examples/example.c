#include<stdlib.h>
#include<stdio.h>
#include<string.h>
#include<getopt.h>


static void usage(const char *progname) {
	printf("usage: %s [options] IN_FILE [OUT_FILE [WORD...]]\n%s", progname,
	       "  IN_FILE\n"
	       "        an input file for this example program\n"
	       "  OUT_FILE\n"
	       "        where we'll put some output\n"
	       "  WORD\n"
	       "        word(s) of interest\n"
	       "  -h  --help\n"
	       "        print this usage and exit\n"
	       "  -b  --block-size <num>  (aliased: --blocksize --bs)\n"
	       "        set the block size, defaults to 12.\n"
	       "      --fav-number <num>\n"
	       "        your favorite number\n"
	       "  -q  --quiet\n"
	       "        disable output\n"
	       "      --name <arg>\n"
	       "        your name\n"
	       );
}

void parse_args(int argc, char **argv, int *block_size, int *fave_number, int *quiet, char* *username, char* *out_file, char* *in_file, char* **words, size_t *words__size) {
	int block_size__isset = 0;
	int fave_number__isset = 0;
	int username__isset = 0;
	static int block_size__default = 12;
	static int fave_number__default = 0xDEADBEEF;
	static char* username__default = "John Smith\0";
	static struct option longopts[] = {
		{"block-size", required_argument, 0, 98},
		{"fav-number", required_argument, 0, 33},
		{"quiet", no_argument, 0, 113},
		{"name", required_argument, 0, 68},
		{"help", 0, 0, 'h'},
		{0, 0, 0, 0}
	};
	int ch;
	while ((ch = getopt_long(argc, argv, "b:qh", longopts, NULL)) != -1) {
		switch (ch) {
		case 98:
			*block_size = atoi(optarg);
			block_size__isset = 1;
			break;
		case 33:
			*fave_number = atoi(optarg);
			fave_number__isset = 1;
			break;
		case 113:
			*quiet = 1;
			break;
		case 68:
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
		*block_size = block_size__default;
	}
	if (!fave_number__isset) {
		*fave_number = fave_number__default;
	}
	if (!username__isset) {
		*username = username__default;
	}

	if (argc-optind < 1) {
		usage(argv[0]);
		exit(1);
	}
	argv += optind;
	argc -= optind;

	*out_file = argv[0];
	argv++;
	argc -= 1;

	if (argc > 0) {
		*in_file = argv[0];
		argv++; argc--;
	}
	if (argc > 0) {
		*words = argv;
		*words__size = argc;
	}
}

int main(int argc, char **argv) {
	int block_size;
	int fave_number;
	int quiet;
	char* username;
	char* out_file;
	char* in_file;
	char* *words;
	size_t words__size;

	parse_args(argc, argv, &block_size, &fave_number, &quiet, &username, &out_file, &in_file, &words, &words__size);

	/* call your code here */
	return 0;
}
