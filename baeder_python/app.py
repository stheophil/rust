from flask import Flask, render_template, request, jsonify
import requests
from bs4 import BeautifulSoup
import re
import os
from datetime import datetime, timedelta
import hashlib
import json

# Directory where pool-detail responses are cached for 7 days
CACHE_DIR = 'cache'

app = Flask(__name__)

DAY_MAP = {
    "Mo": 0, "Montag": 0,
    "Di": 1, "Dienstag": 1,
    "Mi": 2, "Mittwoch": 2,
    "Do": 3, "Donnerstag": 3,
    "Fr": 4, "Freitag": 4,
    "Sa": 5, "Samstag": 5,
    "So": 6, "Sonntag": 6
}

def parse_time_intervals(text):
    intervals = []
    # Regex matches lines like "Mo - Fr: 06:00 - 21:00" or "Sa: 07:00 - 19:00"
    pattern = re.compile(r'((?:(?:Mo(?:ntag)?|Di(?:enstag)?|Mi(?:ttwoch)?|Do(?:nnerstag)?|Fr(?:eitag)?|Sa(?:mstag)?|So(?:nntag)?)(?:\s*[-–]\s*(?:Mo(?:ntag)?|Di(?:enstag)?|Mi(?:ttwoch)?|Do(?:nnerstag)?|Fr(?:eitag)?|Sa(?:mstag)?|So(?:nntag)?))?)(?:\s*[:\-]\s*)(.+)')
    for line in text.splitlines():
        line = line.strip()
        match = pattern.match(line)
        if match:
            days_part, hours_part = match.groups()
            days = []
            # Normalize day part
            days_part = days_part.replace(' ', '')
            if '-' in days_part or '–' in days_part:
                sep = '-' if '-' in days_part else '–'
                parts = days_part.split(sep)
                if len(parts) == 2:
                    start, end = parts
                    start_day = DAY_MAP.get(start[:2])
                    end_day = DAY_MAP.get(end[:2])
                    if start_day is not None and end_day is not None:
                        if start_day <= end_day:
                            days = list(range(start_day, end_day+1))
                        else:
                            days = list(range(start_day, 7)) + list(range(0, end_day+1))
            else:
                day_abbr = days_part[:2]
                day_idx = DAY_MAP.get(day_abbr)
                if day_idx is not None:
                    days = [day_idx]
            if days:
                intervals.append({"days": days, "hours": hours_part.strip(), "length": len(days)})
    return intervals

def get_opening_times_for_7days(intervals):
    results = {}
    today = datetime.now()
    for i in range(7):
        day = today + timedelta(days=i)
        weekday = day.weekday()  # Monday=0, Sunday=6
        applicable = [iv for iv in intervals if weekday in iv["days"]]
        chosen = None
        if applicable:
            # Choose the most specific interval (covering fewer days)
            chosen = sorted(applicable, key=lambda x: x["length"])[0]
        date_str = day.strftime("%Y-%m-%d (%a)")
        results[date_str] = chosen["hours"] if chosen else "Geschlossen"
    return results

def _scrape_pool_detail_old(url):
    try:
        headers = {"User-Agent": "Mozilla/5.0"}
        r = requests.get(url, headers=headers, timeout=10)
        if r.status_code != 200:
            return {}
        soup = BeautifulSoup(r.content, "html.parser")
        text = ""
        # Look for a header containing opening times
        header = soup.find(lambda tag: tag.name in ['h2', 'h3'] and 'Öffnungszeiten' in tag.get_text())
        if header:
            sibling = header.find_next_sibling()
            if sibling:
                text = sibling.get_text(separator='\n')
        if not text:
            # Fallback: search for lines mentioning day abbreviations
            all_text = soup.get_text(separator='\n')
            lines = []
            for line in all_text.splitlines():
                if any(day in line for day in ["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"]):
                    lines.append(line)
            text = "\n".join(lines)
        intervals = parse_time_intervals(text)
        schedule = get_opening_times_for_7days(intervals)
        title_tag = soup.find("h1")
        name = title_tag.get_text().strip() if title_tag else url.split("/")[-1]
        return {"name": name, "schedule": schedule}
    except Exception:
        return {}

def _scrape_main_page_old():
    try:
        headers = {"User-Agent": "Mozilla/5.0"}
        r = requests.get("https://www.berlinerbaeder.de/baeder/", headers=headers, timeout=10)
        if r.status_code != 200:
            return []
        soup = BeautifulSoup(r.content, "html.parser")
        pools = []
        for a in soup.find_all("a", href=True):
            href = a["href"]
            if "/baeder/detail/" in href:
                full_url = "https://www.berlinerbaeder.de" + href if href.startswith("/") else href
                name = a.get_text().strip()
                if not any(p.get("url") == full_url for p in pools):
                    pools.append({"name": name, "url": full_url})
        return pools
    except Exception:
        return []
    
# New scraper implementations replacing old functions
def scrape_main_page():
    try:
        # Cache main page pools for 30 days
        main_cache = os.path.join(CACHE_DIR, 'main_pools.json')
        if os.path.exists(main_cache):
            try:
                mtime = datetime.fromtimestamp(os.path.getmtime(main_cache))
                if datetime.now() - mtime <= timedelta(days=30):
                    return json.load(open(main_cache, encoding='utf-8'))
            except Exception:
                pass
        base_url = 'https://www.berlinerbaeder.de/baeder/'
        headers = {'User-Agent': 'Mozilla/5.0'}
        # Load first page (local baeder.html if available)
        if os.path.exists('baeder.html'):
            with open('baeder.html', encoding='utf-8') as f:
                soup = BeautifulSoup(f.read(), 'html.parser')
        else:
            r = requests.get(base_url, headers=headers, timeout=10)
            if r.status_code != 200:
                return []
            soup = BeautifulSoup(r.content, 'html.parser')
        # Determine which pages to process
        if os.path.exists('baeder.html'):
            # offline/local test: only parse first page
            pages = {1}
        else:
            # detect all pagination pages (including total from mobile indicator)
            pages = {1}
            pagination = soup.find('ul', class_='pagination')
            if pagination:
                # explicit page links
                for a in pagination.find_all('a', href=True):
                    m = re.search(r'/baeder/page/(\d+)/', a['href'])
                    if m:
                        pages.add(int(m.group(1)))
                # total pages from mobile "1 von N"
                mobile_span = pagination.select_one('li.mobile span')
                if mobile_span:
                    m2 = re.search(r'von\s*(\d+)', mobile_span.get_text(strip=True))
                    if m2:
                        total = int(m2.group(1))
                        pages |= set(range(1, total+1))
        pools = []
        seen_urls = set()
        # Iterate through each page
        for page in sorted(pages):
            if page == 1:
                page_soup = soup
            else:
                page_url = f"{base_url}page/{page}/"
                try:
                    r = requests.get(page_url, headers=headers, timeout=10)
                    if r.status_code != 200:
                        continue
                    page_soup = BeautifulSoup(r.content, 'html.parser')
                except Exception:
                    continue
            # Extract pool entries
            for item in page_soup.select('div.bathlist_item'):
                link = item.select_one('a.list_title_link') or item.select_one('a.bathlist_item_image_link')
                if not link:
                    continue
                name = link.get_text(strip=True)
                href = link.get('href', '')
                full_url = href if href.startswith('http') else 'https://www.berlinerbaeder.de' + href
                if full_url in seen_urls:
                    continue
                seen_urls.add(full_url)
                pools.append({'name': name, 'url': full_url})
        # Cache the pools list for next 30 days
        try:
            os.makedirs(CACHE_DIR, exist_ok=True)
            with open(main_cache, 'w', encoding='utf-8') as cf:
                json.dump(pools, cf)
        except Exception:
            pass
        return pools
    except Exception:
        return []

def scrape_pool_detail(url):
    try:
        # Determine local fixture file (offline testing)
        local_file = None
        if os.path.exists('fischer.html') and 'fischerinsel' in url:
            local_file = 'fischer.html'
        elif os.path.exists('sse.html') and 'europasportpark-sse' in url:
            local_file = 'sse.html'
        elif os.path.exists('kreuzberg.html') and 'kreuzberg' in url:
            local_file = 'kreuzberg.html'
        # Initialize for caching raw intervals
        live = (local_file is None)
        cache_path = None
        name = None
        table_intervals = None
        # Attempt to load cached raw intervals if live
        if live:
            try:
                os.makedirs(CACHE_DIR, exist_ok=True)
            except Exception:
                pass
            key = hashlib.md5(url.encode('utf-8')).hexdigest()
            cache_path = os.path.join(CACHE_DIR, f'{key}.json')
            if os.path.exists(cache_path):
                mtime = datetime.fromtimestamp(os.path.getmtime(cache_path))
                if datetime.now() - mtime <= timedelta(days=7):
                    try:
                        data = json.load(open(cache_path, encoding='utf-8'))
                        name = data.get('name')
                        table_intervals = data.get('table_intervals')
                    except Exception:
                        table_intervals = None
        # If no cached intervals, parse HTML and build them
        if table_intervals is None:
            # Load HTML (fixture or live request)
            if local_file:
                with open(local_file, encoding='utf-8') as f:
                    soup = BeautifulSoup(f.read(), 'html.parser')
            else:
                headers = {'User-Agent': 'Mozilla/5.0'}
                r = requests.get(url, headers=headers, timeout=10)
                if r.status_code != 200:
                    return {}
                soup = BeautifulSoup(r.content, 'html.parser')
            # Extract pool name
            name_tag = soup.find('h1')
            name = name_tag.get_text(strip=True) if name_tag else url.rstrip('/').split('/')[-1]
            # Prepare list for raw intervals
            table_intervals = []
        # choose the active area pane dynamically
        area_div = None
        ft_container = soup.find('div', class_='facilitytimes')
        if ft_container:
            area_tabcontent = ft_container.select_one('div.tab-content.tableareas')
            if area_tabcontent:
                active_area = area_tabcontent.select_one('div.tab-pane.show.active')
                if active_area and active_area.has_attr('id'):
                    area_div = active_area
        # fallback to previous hardcoded Hallenbad area
        if not area_div:
            area_div = soup.find('div', id='area-4')
        if area_div:
            for h3 in area_div.find_all('h3'):
                table = h3.find_next_sibling('table', class_='openingtime')
                if not table:
                    continue
                caption = table.find('caption')
                if not caption:
                    continue
                m = re.search(r'(\d{2}\.\d{2}\.\d{2})\s*[-–]\s*(\d{2}\.\d{2}\.\d{2})', caption.get_text())
                if not m:
                    continue
                start_date = datetime.strptime(m.group(1), '%d.%m.%y').date()
                end_date = datetime.strptime(m.group(2), '%d.%m.%y').date()

                last_day_idx = None
                tbody = table.find('tbody') or table
                for elem in tbody.find_all(['tr', 'td'], recursive=False):
                    if elem.name == 'tr':
                        th = elem.find('th')
                        if th:
                            day_name = th.get_text(strip=True)
                            day_idx = DAY_MAP.get(day_name, DAY_MAP.get(day_name[:2]))
                            last_day_idx = day_idx
                        else:
                            day_idx = last_day_idx
                        if day_idx is None:
                            continue
                        td = elem.find('td')
                        if not td:
                            continue
                        title_attr = td.get('title', '') or ''
                        if 'Schul' in title_attr:
                            continue
                        for span in td.select('span.mobileday'):
                            span.extract()
                        raw = td.get_text(separator=' ', strip=True)
                        hours = re.sub(r'\s+', ' ', raw)
                        table_intervals.append({
                            'start': start_date,
                            'end': end_date,
                            'weekday': day_idx,
                            'hours': hours,
                            'range_length': (end_date - start_date).days
                        })
                    elif elem.name == 'td':
                        text = elem.get_text(separator=' ', strip=True)
                        if re.search(r'\d{1,2}:\d{2}', text):
                            title_attr = elem.get('title', '') or ''
                            if 'Schul' in title_attr:
                                continue
                            raw = text
                            hours = re.sub(r'\s+', ' ', raw)
                            table_intervals.append({
                                'start': start_date,
                                'end': end_date,
                                'weekday': last_day_idx,
                                'hours': hours,
                                'range_length': (end_date - start_date).days
                            })

        # Build schedule for next 7 days, including multiple intervals per day
        schedule = {}
        today = datetime.now().date()
        for i in range(7):
            d = today + timedelta(days=i)
            wd = d.weekday()
            # all intervals covering this weekday and date
            matches = [iv for iv in table_intervals if iv['weekday'] == wd and iv['start'] <= d <= iv['end']]
            if matches:
                # pick intervals from the most specific date-range (smallest span)
                min_span = min(iv['range_length'] for iv in matches)
                specific = [iv for iv in matches if iv['range_length'] == min_span]
                # collect unique hours strings preserving order
                hours_list = []
                for iv in specific:
                    h = iv['hours']
                    if h not in hours_list:
                        hours_list.append(h)
                schedule[d.strftime('%Y-%m-%d (%a)')] = '; '.join(hours_list)
            else:
                schedule[d.strftime('%Y-%m-%d (%a)')] = 'Geschlossen'
        # Prepare result and cache raw intervals if live
        result = {'name': name, 'schedule': schedule}
        if live and cache_path:
            try:
                with open(cache_path, 'w', encoding='utf-8') as cf:
                    json.dump({'name': name, 'table_intervals': table_intervals}, cf)
            except Exception:
                pass
        return result
    except Exception:
        return {}

@app.route("/")
def index():
    # Load list of available pools (cached for 30 days)
    pool_list = scrape_main_page()
    return render_template("index.html", pool_list=pool_list)
@app.route("/api/pool_detail")
def api_pool_detail():
    # AJAX endpoint returning pool detail (name and 7-day schedule)
    url = request.args.get('url')
    if not url:
        return jsonify({'error': 'Missing url parameter'}), 400
    detail = scrape_pool_detail(url)
    # include URL for front-end link
    detail['url'] = url
    return jsonify(detail)

if __name__ == "__main__":
    app.run(debug=True)