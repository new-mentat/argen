#include<stdlib.h>
#include<stdio.h>
#include<string.h>
#include<getopt.h>


static void usage(const char *progname) {
	printf("usage: %s [options] IN_FILE [OUT_FILE [WORD...]]\n%s", progname,
	       "  IN_FILE\n"
	       "  OUT_FILE\n"
	       "        the output file\n"
	       "  WORD\n"
	       "        word(s) of interest\n"
	       "  -h  --help\n"
	       "        print this usage and exit\n"
	       "  -b  --block-size  --bs\n"
	       "        this is help text for block_size, defaults to 12.\n"
	       "       --fav-number\n"
	       "        favorite number\n"
	       "  -q  --quiet\n"
	       "       --name\n"
	       );
}

void parse_args(int argc, char **argv, char* *out_file, char* *in_file, char* **words, size_t *words__size, int *block_size, int *fave_number, int *quiet, char* *username) {
	int block_size__isset = 0;
	int fave_number__isset = 0;
	int username__isset = 0;
	static struct option longopts[] = {
		{"block-size", required_argument, 0, 98},
		{"fav-number", required_argument, 0, 166},
		{"quiet", no_argument, 0, 113},
		{"name", required_argument, 0, 34},
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
		case 166:
			*fave_number = atoi(optarg);
			fave_number__isset = 1;
			break;
		case 113:
			*quiet = 1;
			break;
		case 34:
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
		*block_size = 12;
	}
	if (!fave_number__isset) {
		*fave_number = 0xDEADBEEF;
	}
	if (!username__isset) {
		*username = "John Smith";
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
	char* out_file;
	char* in_file;
	char* *words;
	size_t words__size;
	int block_size;
	int fave_number;
	int quiet;
	char* username;

	parse_args(argc, argv, &out_file, &in_file, &words, &words__size, &block_size, &fave_number, &quiet, &username);

	/* call your code here */
}
