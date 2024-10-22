#!/usr/bin/env python3
# coding=utf-8
import argparse
import ctypes
import os
import re
import stat
import subprocess
import sys
import time
from collections import namedtuple
from math import gcd
from pprint import pprint


class UserError(Exception):
    def __init__(self, message):
        self.message = message


CHAR_ENCODING = 'UTF-8'
ENCODE_ALGO = 'x264'  # or x264


def check_err(*popenargs, **kwargs):
    process = subprocess.Popen(stderr=subprocess.PIPE, *popenargs, **kwargs)
    _, stderr = process.communicate()
    retcode = process.poll()
    if retcode:
        cmd = kwargs.get("args")
        if cmd is None:
            cmd = popenargs[0]
        raise subprocess.CalledProcessError(retcode, cmd, output=stderr)
    return stderr.decode(CHAR_ENCODING, 'replace')


def check_output(*args, **kwargs):
    s = subprocess.check_output(*args, **kwargs).decode(CHAR_ENCODING)
    return s.replace(os.linesep, '\n')


HANDBRAKE = 'HandBrakeCLI'

TITLE_COUNT_REGEXES = [
    re.compile(r'^Scanning title \d+ of (\d+)\.\.\.$'),
    re.compile(r'^\[\d\d:\d\d:\d\d] scan: DVD has (\d+) title\(s\)$'),
]


def FindTitleCount(scan, verbose):
    for regex in TITLE_COUNT_REGEXES:
        for line in scan:
            m = regex.match(line)
            if m: break
        if m:
            return int(m.group(1))
    if verbose:
        for line in scan:
            print(line)
    raise AssertionError("Can't find TITLE_COUNT_REGEX in scan")


STRUCTURED_LINE_RE = re.compile(r'( *)\+ (([a-z0-9 ]+):)?(.*)')


def ExtractTitleScan(scan):
    result = []
    in_title_scan = False
    for line in scan:
        if not in_title_scan:
            if line.startswith('+'):
                in_title_scan = True
        if in_title_scan:
            m = STRUCTURED_LINE_RE.match(line)
            if m:
                result.append(line)
            else:
                break
    return tuple(result)


TRACK_VALUE_RE = re.compile(r'(\d+), (.*)')


def MassageTrackData(node, key):
    if key in node:
        track_data = node[key]
        if type(track_data) is list:
            new_track_data = {}
            for track in track_data:
                k, v = TRACK_VALUE_RE.match(track).groups()
                new_track_data[k] = v
            node[key] = new_track_data


def ParseTitleScan(scan):
    pos, result = ParseTitleScanHelper(scan, pos=0, indent=0)
    for value in result.values():
        MassageTrackData(value, 'audio tracks')
        MassageTrackData(value, 'subtitle tracks')
    return result


def ParseTitleScanHelper(scan, pos, indent):
    result = {}
    cruft = []
    while True:
        pos, node = ParseNode(scan, pos=pos, indent=indent)
        if node:
            if type(node) is tuple:
                k, v = node
                result[k] = v
            else:
                cruft.append(node)
                result[None] = cruft
        else:
            break
    if len(result) == 1 and None in result:
        result = result[None]
    return pos, result


def ParseNode(scan, pos, indent):
    if pos >= len(scan):
        return pos, None
    line = scan[pos]
    spaces, colon, name, value = STRUCTURED_LINE_RE.match(line).groups()
    spaces = len(spaces) / 2
    if spaces < indent:
        return pos, None
    assert spaces == indent, '%d <> %r' % (indent, line)
    pos += 1
    if colon:
        if value:
            node = (name, value)
        else:
            pos, children = ParseTitleScanHelper(scan, pos, indent + 1)
            node = (name, children)
    else:
        node = value
    return pos, node


def only(iterable):
    result, = iterable
    return result


Title = namedtuple('Title', ['number', 'info'])
Task = namedtuple('Task', ['title', 'chapter'])

TOTAL_EJECT_SECONDS = 5
EJECT_ATTEMPTS_PER_SECOND = 10


class DVD:
    def __init__(self, mountpoint, verbose, mount_timeout=0):
        if stat.S_ISBLK(os.stat(mountpoint).st_mode):
            mountpoint = FindMountPoint(mountpoint, mount_timeout)
        if not os.path.isdir(mountpoint):
            raise UserError('%r is not a directory' % mountpoint)
        self.mountpoint = mountpoint
        self.verbose = verbose

    def RipTitle(self, task, output, dry_run, verbose):
        if verbose:
            print('Title Scan:')
            pprint(task.title.info)
            print('-' * 78)

        audio_tracks = task.title.info['audio tracks'].keys()
        audio_encoders = ['faac'] * len(audio_tracks)
        subtitles = task.title.info['subtitle tracks'].keys()

        print(f"Using {ENCODE_ALGO} for encoding. (track: {task.title.number})")
        args = [
            HANDBRAKE,
            '--title', str(task.title.number),
            '--preset', "Production Standard",
            '--encoder', ENCODE_ALGO,
            '--audio', ','.join(audio_tracks),
            '--aencoder', ','.join(audio_encoders),
        ]
        if task.chapter is not None:
            args += [
                '--chapters', str(task.chapter),
            ]
        if subtitles:
            args += [
                '--subtitle', ','.join(subtitles),
            ]
        args += [
            '--markers',
            '--optimize',
            '--input', self.mountpoint,
            '--output', output,
        ]
        if verbose:
            print(' '.join(('\n  ' + a)
                           if a.startswith('-') else a for a in args))
            print('-' * 78)
        if not dry_run:
            if verbose:
                subprocess.call(args)
            else:
                check_err(args)

    def ScanTitle(self, i):
        for line in check_err([
            HANDBRAKE,
            '--scan',
            '--title', str(i),
            '-i',
            self.mountpoint], stdout=subprocess.PIPE).split(os.linesep):
            if self.verbose:
                print('< %s' % line.rstrip())
            yield line

    def ScanTitles(self, title_numbers, verbose):
        first = title_numbers[0] if title_numbers else 1
        try:
            raw_scan = tuple(self.ScanTitle(first))
        except subprocess.CalledProcessError:
            retries = 0
            print('Failed to scan title %d, trying other titles.' % first)
            while True:
                if retries >= 10:
                    raise UserError('Failed to scan titles.')
                try:
                    first += 1
                    raw_scan = tuple(self.ScanTitle(first))
                    print('Scanned title %d successfully.' % first)
                    break
                except subprocess.CalledProcessError:
                    print('Failed to scan title %d, trying other titles.' % first)
                    retries += 1
                    time.sleep(1)

        title_count = FindTitleCount(raw_scan, verbose)
        print('Disc claims to have %d titles.' % title_count)
        title_name, title_info = only(
            ParseTitleScan(ExtractTitleScan(raw_scan)).items())
        del raw_scan

        def MakeTitle(name, number, info):
            assert ('title %d' % number) == name
            info['duration'] = ExtractDuration('duration ' + info['duration'])
            return Title(number, info)

        yield MakeTitle(title_name, first, title_info)

        to_scan = [x for x in range(1, title_count + 1)
                   if x != first
                   and ((not title_numbers)
                        or x in title_numbers)]
        for i in to_scan:
            try:
                scan = ExtractTitleScan(self.ScanTitle(i))
            except subprocess.CalledProcessError as exc:
                warn("Cannot scan title %d." % i)
            else:
                title_info_names = ParseTitleScan(scan).items()
                if title_info_names:
                    title_name, title_info = only(title_info_names)
                    yield MakeTitle(title_name, i, title_info)
                else:
                    warn("Cannot parse scan of title %d." % i)

    def Eject(self):
        if os.name == 'nt':
            if len(self.mountpoint) < 4 and self.mountpoint[1] == ':':
                # mountpoint is only a drive letter like "F:" or "F:\" not a subdirectory
                drive_letter = self.mountpoint[0]
                ctypes.windll.WINMM.mciSendStringW(
                    "open %s: type CDAudio alias %s_drive" % (drive_letter, drive_letter), None, 0, None)
                ctypes.windll.WINMM.mciSendStringW("set %s_drive door open" % drive_letter, None, 0, None)
            return
        return
        """for i in range(TOTAL_EJECT_SECONDS * EJECT_ATTEMPTS_PER_SECOND):
            if not subprocess.call(['eject', self.mountpoint]):
                return
            time.sleep(1.0 / EJECT_ATTEMPTS_PER_SECOND)"""


def ParseDuration(s):
    result = 0
    for field in s.strip().split(':'):
        result *= 60
        result += int(field)
    return result


def FindMountPoint(dev, timeout):
    regex = re.compile(r'^' + re.escape(os.path.realpath(dev)) + r'\b')

    now = time.time()
    end_time = now + timeout
    while end_time >= now:
        for line in check_output(['df', '-P']).split('\n'):
            m = regex.match(line)
            if m:
                line = line.split(None, 5)
                if len(line) > 1:
                    return line[-1]
        time.sleep(0.1)
        now = time.time()
    raise UserError('%r not mounted.' % dev)


def FindMainFeature(titles, verbose=False):
    if verbose:
        print('Attempting to determine main feature of %d titles...'
              % len(titles))
    main_feature = max(titles,
                       key=lambda title: ParseDuration(title.info['duration']))
    if verbose:
        print('Selected %r as main feature.' % main_feature.number)
        print()


def ConstructTasks(titles, chapter_split):
    for title in titles:
        num_chapters = len(title.info['chapters'])
        if chapter_split and num_chapters > 1:
            for chapter in range(1, num_chapters + 1):
                yield Task(title, chapter)
        else:
            yield Task(title, None)


def TaskFilenames(tasks, output, dry_run=False):
    if (len(tasks) > 1):
        def ComputeFileName(task):
            if task.chapter is None:
                return os.path.join(output,
                                    'Title%02d.mp4' % task.title.number)
            else:
                return os.path.join(output,
                                    'Title%02d_%02d.mp4'
                                    % (task.title.number, task.chapter))

        if not dry_run:
            os.makedirs(output)
    else:
        def ComputeFileName(task):
            return '%s.mp4' % output
    result = [ComputeFileName(task) for task in tasks]
    if len(set(result)) != len(result):
        raise UserError("multiple tasks use same filename")
    return result


def PerformTasks(dvd, tasks, title_count, filenames,
                 dry_run=False, verbose=False):
    for task, filename in zip(tasks, filenames):
        print('=' * 78)
        if task.chapter is None:
            print('Title %s / %s => %r'
                  % (task.title.number, title_count, filename))
        else:
            num_chapters = len(task.title.info['chapters'])
            print('Title %s / %s , Chapter %s / %s=> %r'
                  % (task.title.number, title_count, task.chapter,
                     num_chapters, filename))
        print('-' * 78)
        dvd.RipTitle(task, filename, dry_run, verbose)


Size = namedtuple('Size',
                  ['width', 'height', 'pix_aspect_width', 'pix_aspect_height', 'fps'])

SIZE_REGEX = re.compile(
    r'^\s*(\d+)x(\d+),\s*'
    r'pixel aspect: (\d+)/(\d+),\s*'
    r'display aspect: (?:\d+(?:\.\d+)),\s*'
    r'(\d+(?:\.\d+)) fps\s*$')

SIZE_CTORS = [int] * 4 + [float]


def ParseSize(s):
    return Size(*(f(x)
                  for f, x in zip(SIZE_CTORS, SIZE_REGEX.match(s).groups())))


def ComputeAspectRatio(size):
    w = size.width * size.pix_aspect_width
    h = size.height * size.pix_aspect_height
    d = gcd(w, h)
    return (w // d, h // d)


DURATION_REGEX = re.compile(
    r'^(?:.*,)?\s*duration\s+(\d\d):(\d\d):(\d\d)\s*(?:,.*)?$')


class Duration(namedtuple('Duration', 'hours minutes seconds')):
    def __str__(self):
        return '%02d:%02d:%02d' % (self)

    def in_seconds(self):
        return 60 * (60 * self.hours + self.minutes) + self.seconds


def ExtractDuration(s):
    return Duration(*map(int, DURATION_REGEX.match(s).groups()))


Chapter = namedtuple('Chapter', 'number duration')


def ParseChapters(d):
    for number, info in sorted(((int(n), info) for (n, info) in d.items())):
        yield Chapter(number, ExtractDuration(info))


AUDIO_TRACK_REGEX = re.compile(
    r'^(\S+)\s*((?:\([^)]*\)\s*)*)(?:,\s*(.*))?$')

AUDIO_TRACK_FIELD_REGEX = re.compile(
    r'^\(([^)]*)\)\s*\(([^)]*?)\s*ch\)\s*' +
    r'((?:\([^()]*\)\s*)*)\(iso639-2:\s*([^)]+)\)$')

AudioTrack = namedtuple('AudioTrack',
                        'number lang codec channels iso639_2 extras')


def ParseAudioTracks(d):
    for number, info in sorted(((int(n), info) for (n, info) in d.items())):
        m = AUDIO_TRACK_REGEX.match(info)
        if m:
            lang, field_string, extras = m.groups()
            m2 = AUDIO_TRACK_FIELD_REGEX.match(field_string)
            if m2:
                codec, channels, more_extras, iso639_2 = m2.groups()
                if more_extras:
                    extras = more_extras + extras
                yield AudioTrack(number, lang, codec, channels,
                                 iso639_2, extras)
            else:
                warn('Cannot parse audio track fields %r' % field_string)
        else:
            warn('Cannot parse audio track info %r' % info)


SubtitleTrack = namedtuple('SubtitleTrack',
                           'number info')


def ParseSubtitleTracks(d):
    for number, info in sorted(((int(n), info) for (n, info) in d.items())):
        yield SubtitleTrack(number, info)


def RenderBar(start, length, total, width):
    end = start + length
    start = int(round(start * (width - 1) / total))
    length = int(round(end * (width - 1) / total)) - start + 1
    return ('‥' * start +
            '■' * length +
            '‥' * (width - start - length))


MAX_BAR_WIDTH = 50


def DisplayScan(titles):
    max_title_seconds = max(
        title.info['duration'].in_seconds()
        for title in titles)

    for title in titles:
        info = title.info
        size = ParseSize(info['size'])
        xaspect, yaspect = ComputeAspectRatio(size)
        duration = info['duration']
        title_seconds = duration.in_seconds()
        print('Title % 3d/% 3d: %s  %d×%d  %d:%d  %3g fps' %
              (title.number, len(titles), duration, size.width,
               size.height, xaspect, yaspect, size.fps))
        for at in ParseAudioTracks(info['audio tracks']):
            print('  audio % 3d: %s (%sch)  [%s]' %
                  (at.number, at.lang, at.channels, at.extras))
        for sub in ParseSubtitleTracks(info['subtitle tracks']):
            print('  sub % 3d: %s' %
                  (sub.number, sub.info))
        position = 0
        if title_seconds > 0:
            for chapter in ParseChapters(info['chapters']):
                seconds = chapter.duration.in_seconds()
                bar_width = int(round(
                    MAX_BAR_WIDTH * title_seconds / max_title_seconds))
                bar = RenderBar(position, seconds, title_seconds, bar_width)
                print('  chapter % 3d: %s ◖%s◗'
                      % (chapter.number, chapter.duration, bar))
                position += seconds
        print()


def ParseArgs():
    parser = argparse.ArgumentParser(formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('-v', '--verbose',
                        action='store_true')
    parser.add_argument('-c', '--chapter_split',
                        action='store_true')
    parser.add_argument('-n', '--dry-run',
                        action='store_true')
    parser.add_argument('--scan',
                        action='store_true')
    parser.add_argument('--main-feature',
                        action='store_true')
    parser.add_argument('-t', '--titles',
                        default="*")
    parser.add_argument('-i', '--input', required=True)
    parser.add_argument('-o', '--output')
    parser.add_argument('--mount-timeout',
                        default=15, type=float)
    args = parser.parse_args()
    if not args.scan and args.output is None:
        raise UserError("output argument is required")
    return args


NUM_RANGE_REGEX = re.compile(r'^(\d*)-(\d+)|(\d+)$')


def parse_titles_arg(titles_arg):
    if titles_arg == '*':
        return None  # all titles
    else:
        def str_to_ints(s):
            m = NUM_RANGE_REGEX.match(s)
            if not m:
                raise UserError(
                    "--titles must be * or list of integer ranges, found %r" %
                    titles_arg)
            else:
                start, end, only = m.groups()
                if only is not None:
                    return [int(only)]
                else:
                    start = int(start) if start else 1
                    end = int(end)
                    return range(start, end + 1)

        result = set()
        for s in titles_arg.split(','):
            result.update(str_to_ints(s))
        result = sorted(list(result))
        return result


def main():
    args = ParseArgs()
    dvd = DVD(args.input, args.verbose, args.mount_timeout)
    print('Reading from %r' % dvd.mountpoint)
    title_numbers = parse_titles_arg(args.titles)
    titles = tuple(dvd.ScanTitles(title_numbers, args.verbose))

    if args.scan:
        DisplayScan(titles)
    else:
        if args.main_feature and len(titles) > 1:
            titles = [FindMainFeature(titles, args.verbose)]

        if not titles:
            raise UserError("No titles to rip")
        else:
            if not args.output:
                raise UserError("No output specified")
            print('Writing to %r' % args.output)
            tasks = tuple(ConstructTasks(titles, args.chapter_split))

            filenames = TaskFilenames(tasks, args.output, dry_run=args.dry_run)
            # Don't stomp on existing files
            for filename in filenames:
                if os.path.exists(filename):
                    raise UserError('%r already exists' % filename)

            PerformTasks(dvd, tasks, len(titles), filenames,
                         dry_run=args.dry_run, verbose=args.verbose)

            print('=' * 78)
            if not args.dry_run:
                dvd.Eject()


def warn(msg):
    print('warning: %s' % (msg,), file=sys.stderr)


if __name__ == '__main__':
    error = None
    try:
        main()
    except FileExistsError as exc:
        error = '%s: %r' % (exc.strerror, exc.filename)
    except UserError as exc:
        error = exc.message

    if error is not None:
        print('%s: error: %s'
              % (os.path.basename(sys.argv[0]), error), file=sys.stderr)
        sys.exit(1)
